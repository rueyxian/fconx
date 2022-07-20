use crate::config::Series;
use crate::episode::Episode;
use crate::rw::RWJson;

///
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

///
pub struct Scraper {
    max_worker: usize,
    series_vec: std::sync::Arc<Vec<Series>>,
    rw_json: std::sync::Arc<RWJson>,
}

///
impl Scraper {
    ///
    pub fn new(
        max_worker: usize,
        series_vec: std::sync::Arc<Vec<Series>>,
        rw_json: std::sync::Arc<RWJson>,
    ) -> Scraper {
        Scraper {
            max_worker,
            series_vec,
            rw_json,
        }
    }

    ///
    pub async fn run(self) -> Result<()> {
        println!("================ scan unscraped episode ================");
        let to_scrape = self.scan_unscraped_episodes().await?;
        println!("{} episodes to scrape", to_scrape.len());
        println!("================ start scraping  ================");
        self.scrape_download_urls(to_scrape).await?;
        println!("================ done scraping  ================");
        Ok(())
    }

    ///
    async fn scan_unscraped_episodes(&self) -> Result<Vec<Episode>> {
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
                let scraped = json_rw.read_all_episodes(&series).unwrap();
                let all = Scraper::scrape_page_urls(series).await?;

                let mut to_scraped = all
                    .into_iter()
                    .filter(|episode| !scraped.iter().any(|scraped| scraped.id() == episode.id()))
                    .collect::<Vec<Episode>>();
                {
                    let mut out_guard = out_mutex.lock();
                    let out = out_guard.as_mut().unwrap();
                    out.append(&mut to_scraped);
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
    async fn scrape_page_urls(series: Series) -> Result<Vec<Episode>> {
        let browser = {
            let opts = headless_chrome::LaunchOptionsBuilder::default()
                // .headless(false)
                .idle_browser_timeout(std::time::Duration::from_millis(10_000))
                .build()?;
            headless_chrome::Browser::new(opts)?
        };

        let tab = browser.wait_for_initial_tab()?;
        tab.navigate_to(&series.url().to_string())?;
        let elems = tab.wait_for_elements(".archive_entry").unwrap();

        let mut out = Vec::<Episode>::with_capacity(elems.len());

        let episode_num_sel = scraper::Selector::parse(".episode_number a").unwrap();
        let entry_content_sel = scraper::Selector::parse(".entry_content a").unwrap();
        let date_sel = scraper::Selector::parse(".date").unwrap();

        for elem in elems {
            let html = {
                // https://github.com/atroche/rust-headless-chrome/issues/73
                let remote_obj = elem
                    .call_js_fn("function() { return this.innerHTML; }", true)
                    .unwrap();
                let html_str = remote_obj.value.unwrap().to_owned().to_string();
                let html_str = html_str.replace("\\\"", "\"");
                scraper::Html::parse_fragment(html_str.as_str())
            };

            let number = {
                if let Some(elem_ref) = html.select(&episode_num_sel).next() {
                    let number_raw = elem_ref.text().next().unwrap();
                    let number_str = number_raw.trim_start_matches("No. ");
                    if let Ok(number) = number_str.parse::<usize>() {
                        format!("{:04}", number)
                    } else {
                        number_str.to_string()
                    }
                } else {
                    continue;
                }
            };

            let (title, page_url) = {
                if let Some(elem_ref) = html.select(&entry_content_sel).next() {
                    let title = elem_ref.text().next().unwrap();
                    let url = elem_ref.value().attr("href").unwrap();
                    (title.to_string(), url.to_string())
                } else {
                    continue;
                }
            };

            let date = {
                if let Some(elem_ref) = html.select(&date_sel).next() {
                    let date_str = elem_ref.text().next().unwrap();
                    chrono::NaiveDate::parse_from_str(&date_str, "%m/%d/%y").unwrap()
                } else {
                    continue;
                }
            };

            let episode = Episode::new(series, number, title, date, page_url);
            out.push(episode);
        }

        Ok(out)
    }

    ///
    pub async fn scrape_download_urls(&self, episodes: Vec<Episode>) -> Result<()> {
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
                        if let Err(err) = Scraper::scrape_download_url(&mut episode).await {
                            println!(
                                "SCRAPE ERROR: {:?} {} {:?}: {:?}",
                                episode.series(),
                                episode.number(),
                                episode.title(),
                                err,
                            );
                            continue;
                        }

                        rw_json.push_episode(episode).unwrap();
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
            let truncated = url.get(..idx)?;
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

