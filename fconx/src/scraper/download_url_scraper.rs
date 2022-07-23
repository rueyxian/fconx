use crate::canceller::Canceller;
use crate::config::Series;
use crate::episode::Episode;
use crate::logger::Logger;
use crate::rw::RWJson;

///
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

///
pub(crate) struct DownloadUrlScraper {
    logger: std::sync::Arc<Logger>,
    canceller: std::sync::Arc<Canceller>,
    thread_max: usize,
    series_vec: std::sync::Arc<Vec<Series>>,
    rw_json: std::sync::Arc<RWJson>,
}

///
impl DownloadUrlScraper {
    ///
    pub(crate) fn new(
        logger: std::sync::Arc<Logger>,
        canceller: std::sync::Arc<Canceller>,
        thread_max: usize,
        series_vec: std::sync::Arc<Vec<Series>>,
        rw_json: std::sync::Arc<RWJson>,
    ) -> DownloadUrlScraper {
        DownloadUrlScraper {
            logger,
            canceller,
            thread_max,
            series_vec,
            rw_json,
        }
    }

    ///
    pub(crate) async fn run(&self) -> Result<()> {
        let no_download_url_episodes = self.no_download_url_episodes().await?;
        self.logger.log_to_scrape(&no_download_url_episodes).await;
        self.scrape_and_write_download_urls(no_download_url_episodes)
            .await?;
        Ok(())
    }

    ///
    async fn no_download_url_episodes(&self) -> Result<Vec<Episode>> {
        let out_mutex = {
            let v = Some(Vec::<Episode>::new());
            std::sync::Arc::new(parking_lot::Mutex::new(v))
        };
        let mut handles =
            Vec::<tokio::task::JoinHandle<Result<()>>>::with_capacity(self.series_vec.len());
        for &series in self.series_vec.iter() {
            let out_mutex = std::sync::Arc::clone(&out_mutex);
            let json_rw = self.rw_json.arc_clone();
            let h = tokio::spawn(async move {
                let from_json = json_rw.read_all_episodes(&series).unwrap();
                let mut no_download_url = from_json
                    .into_iter()
                    .filter(|episode| episode.download_url().is_none())
                    .collect::<Vec<Episode>>();

                {
                    let mut out_guard = out_mutex.lock();
                    let out = out_guard.as_mut().unwrap();
                    out.append(&mut no_download_url);
                }

                Ok::<(), Box<dyn std::error::Error + Send + Sync + 'static>>(())
            });
            handles.push(h);
        }
        for h in handles {
            h.await.unwrap()?;
        }

        let out = {
            let mut out_guard = out_mutex.lock();
            out_guard.take().unwrap()
        };
        Ok(out)
    }

    ///
    async fn scrape_and_write_download_urls(&self, episodes: Vec<Episode>) -> Result<()> {
        // TODO parking_lot::Mutex or tokio::sync::Mutex?????
        let episodes_count = episodes.len();
        let episodes_mutex = std::sync::Arc::new(tokio::sync::Mutex::new(episodes));
        let mut handles_out = Vec::<tokio::task::JoinHandle<()>>::with_capacity(self.thread_max);

        for idx in 0..usize::min(self.thread_max, episodes_count) {
            let logger = self.logger.arc_clone();
            let canceller = self.canceller.arc_clone();
            let episodes_mutex = std::sync::Arc::clone(&episodes_mutex);
            let rw_json = self.rw_json.arc_clone();
            let h = tokio::spawn(async move {
                loop {
                    if canceller.is_cancel() {
                        break;
                    }
                    let mut episode = {
                        let mut episodes_guard = episodes_mutex.lock().await;
                        let episode = episodes_guard.pop();
                        drop(episodes_guard); // drop the guard immediately
                        match episode {
                            Some(episode) => episode,
                            None => break,
                        }
                    };

                    logger.log_scrape_download_url_start(idx, &episode).await;

                    let maybe_err = DownloadUrlScraper::scrape_download_url(&mut episode).await;

                    if let Err(err) = maybe_err {
                        logger.log_scrape_download_url_error(idx, &episode).await;
                        continue;
                    }

                    rw_json.edit_episode(&episode).unwrap();
                    logger.log_scrape_download_url_done(idx, &episode).await;
                }
                logger.log_scrape_download_url_thread_kill(idx).await;
            });
            handles_out.push(h);
        }

        for h in handles_out {
            h.await.unwrap();
        }

        Ok(())
    }

    ///
    async fn scrape_download_url(episode: &mut Episode) -> Result<()> {
        let browser = {
            let opts = headless_chrome::LaunchOptionsBuilder::default()
                // .headless(false)
                .idle_browser_timeout(std::time::Duration::from_millis(8_000))
                .build()?;
            headless_chrome::Browser::new(opts)?
        };

        let tab = browser.wait_for_initial_tab()?;

        tab.navigate_to(episode.page_url())?;

        fn try_truncate_url(url: &str) -> Option<String> {
            let idx = url.find(".mp3")?;
            let truncated = url.get(..idx + 4)?;
            Some(truncated.to_string())
        }

        let download_url = {
            let elem = tab.wait_for_element(".download > a")?;
            let attrs = elem.get_attributes().unwrap().unwrap();
            let url = attrs.get("href").unwrap();

            match try_truncate_url(url) {
                Some(truncated) => truncated,
                None => url.to_string(),
            }
        };

        episode.set_download_url(download_url);

        Ok(())
    }
}
