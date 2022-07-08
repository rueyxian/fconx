use anyhow::anyhow;
use anyhow::Result;
use futures::StreamExt;

use crate::config::Series;
use crate::episode::Episode;
use crate::rw::RWJson;

///
pub struct Scraper {
    max_worker: usize,
    series_vec: std::rc::Rc<Vec<Series>>,
    rw_json: std::sync::Arc<RWJson>,
}

///
impl Scraper {
    ///
    pub fn new(
        max_worker: usize,
        series_vec: std::rc::Rc<Vec<Series>>,
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
        let episodes = self.scan_unscraped_episodes().await?;
        self.scrape(episodes).await?;
        Ok(())
    }

    ///
    async fn scan_unscraped_episodes(&self) -> Result<Vec<Episode>> {
        let out_mutex = {
            let v = Some(Vec::<Episode>::new());
            std::sync::Arc::new(parking_lot::Mutex::new(v))
        };
        let mut handles = Vec::<tokio::task::JoinHandle<Result<(), anyhow::Error>>>::with_capacity(
            self.series_vec.len(),
        );
        for &series in self.series_vec.iter() {
            let out_mutex = std::sync::Arc::clone(&out_mutex);
            let json_rw = self.rw_json.arc_clone();
            let h = tokio::spawn(async move {
                let scraped = json_rw.read_all_episode(&series).unwrap();
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

                Ok::<(), anyhow::Error>(())
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
    async fn scrape_download_url(episode: &mut Episode) -> Result<()> {
        let config = chromiumoxide::BrowserConfig::builder()
            // .with_head()
            .build()
            .map_err(|e| anyhow!(e))?;
        let (browser, mut handler) = chromiumoxide::Browser::launch(config)
            .await
            .map_err(|e| anyhow!(e))?;

        let addr = browser.websocket_address();
        let port = &addr[15..addr.find("/devtools").unwrap()];
        println!(
            "port {} scraping {:?} {}",
            port,
            episode.series(),
            episode.page_url()
        );

        let close_handler = std::sync::Arc::new(parking_lot::Mutex::new(false));
        let handle = {
            let close_handler = std::sync::Arc::clone(&close_handler);
            async_std::task::spawn(async move {
                loop {
                    if *close_handler.lock() == true {
                        break;
                    }
                    let _event = handler.next().await.unwrap();
                }
            })
        };

        let page = browser.new_page(episode.page_url()).await?;

        let download_url = {
            let url = page
                .find_element(".download>a")
                .await?
                .attribute("href")
                .await?
                .unwrap();
            // let truncated_url = &url[..url.rfind(".mp3").unwrap() + 4];
            let truncated_url = &url[..url.find(".mp3").unwrap() + 4];
            truncated_url.to_string()
        };

        episode.set_download_url(download_url);

        drop(browser);
        *close_handler.lock() = true;
        handle.await;

        Ok(())
    }

    ///
    pub async fn scrape(&self, episodes: Vec<Episode>) -> Result<()> {
        // TODO parking_lot::Mutex or tokio::sync::Mutex?????
        let episodes_mutex = std::sync::Arc::new(tokio::sync::Mutex::new(episodes));
        let mut handles_out = Vec::<tokio::task::JoinHandle<()>>::with_capacity(self.max_worker);

        for _ in 0..self.max_worker {
            let episodes_mutex = std::sync::Arc::clone(&episodes_mutex);
            let rw_json = self.rw_json.arc_clone();
            let h = tokio::spawn(async move {
                loop {
                    let episode = {
                        episodes_mutex.lock().await.pop() // drop the guard immediately
                    };
                    if let Some(mut episode) = episode {
                        while let Err(err) = Scraper::scrape_download_url(&mut episode).await {
                            println!("SCRAPE FAIL: {:?}: {:?}", err, episode.page_url());
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
    async fn scrape_page_urls(series: Series) -> Result<Vec<Episode>> {
        // TODO refactor the code: make a Chrome struct that scrape download url
        // Use channel to pass Episode around instead of Arc + Mutex
        let config = chromiumoxide::BrowserConfig::builder()
            // .with_head()
            .build()
            .map_err(|e| anyhow!(e))?;
        let (browser, mut handler) = chromiumoxide::Browser::launch(config)
            .await
            .map_err(|e| anyhow!(e))?;

        let close_handler = std::sync::Arc::new(parking_lot::Mutex::new(false));
        let handle = {
            let close_handler = std::sync::Arc::clone(&close_handler);
            async_std::task::spawn(async move {
                loop {
                    if *close_handler.lock() == true {
                        break;
                    }
                    let _event = handler.next().await.unwrap();
                }
            })
        };

        let page = browser.new_page(series.url()).await?;
        let elems = page.find_elements(".archive_entry").await?;
        let mut out = Vec::<Episode>::with_capacity(elems.len());

        for elem in elems {
            let number = if let Ok(a) = elem.find_element(".episode_number a").await {
                let number_raw = a.inner_text().await.unwrap().unwrap();
                let number_str = number_raw.trim_start_matches("NO. ");
                if let Ok(number) = number_str.parse::<usize>() {
                    format!("{:04}", number)
                } else {
                    number_str.to_string()
                }
            } else {
                continue;
            };

            let (title, page_url) = if let Ok(a) = elem.find_element(".entry_content a").await {
                let title = a.inner_text().await.unwrap().unwrap();
                let url = a.attribute("href").await.unwrap().unwrap();
                (title, url)
            } else {
                continue;
            };

            let date = if let Ok(div) = elem.find_element(".date").await {
                let date_str = div.inner_text().await.unwrap().unwrap();
                chrono::NaiveDate::parse_from_str(&date_str, "%m/%d/%y").unwrap()
            } else {
                continue;
            };

            let episode = Episode::new(series, number, title, date, page_url);
            out.push(episode);
        }

        drop(browser);
        *close_handler.lock() = true;
        handle.await;

        Ok(out)
    }
}
