use crate::config::Series;
use crate::episode::Episode;
use crate::rw::RWJson;

///
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

///
pub(crate) struct DownloadUrlScraper {
    //
    max_worker: usize,
    series_vec: std::sync::Arc<Vec<Series>>,
    rw_json: std::sync::Arc<RWJson>,
}

impl DownloadUrlScraper {
    ///
    pub(crate) fn new(
        max_worker: usize,
        series_vec: std::sync::Arc<Vec<Series>>,
        rw_json: std::sync::Arc<RWJson>,
    ) -> DownloadUrlScraper {
        DownloadUrlScraper {
            max_worker,
            series_vec,
            rw_json,
        }
    }

    pub(crate) async fn run(&self) -> Result<()> {
        let no_download_url_episodes = self.no_download_url_episodes().await?;
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
        let mut handles_out = Vec::<tokio::task::JoinHandle<()>>::with_capacity(self.max_worker);

        for _ in 0..usize::min(self.max_worker, episodes_count) {
            let episodes_mutex = std::sync::Arc::clone(&episodes_mutex);
            let rw_json = self.rw_json.arc_clone();
            let h = tokio::spawn(async move {
                loop {
                    let episode = {
                        episodes_mutex.lock().await.pop() // drop the guard immediately
                    };
                    if let Some(mut episode) = episode {
                        if let Err(err) =
                            DownloadUrlScraper::scrape_download_url(&mut episode).await
                        {
                            println!(
                                "SCRAPE ERROR: {:?} {} {:?}: {:?}",
                                episode.series(),
                                episode.number(),
                                episode.title(),
                                err,
                            );
                            continue;
                        }

                        // rw_json.push_episode(episode).unwrap();
                        rw_json.edit_episode(episode).unwrap();
                    } else {
                        break;
                    }
                }
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
        let browser = headless_chrome::Browser::default()?;

        let tab = browser.wait_for_initial_tab()?;

        tab.navigate_to(episode.page_url())?;

        println!(
            "scraping: {:?} {} {}",
            episode.series(),
            episode.number(),
            episode.page_url()
        );

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
