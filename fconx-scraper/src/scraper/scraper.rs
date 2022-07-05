use anyhow::anyhow;
use anyhow::Result;
use futures::StreamExt;

use crate::config::Series;
use crate::episode::Episode;
use crate::rw::RWJson;

// ===============================================================================================
//

pub struct Scraper {
    max_worker: usize,
    series_vec: std::rc::Rc<Vec<Series>>,
    rw_json: std::sync::Arc<RWJson>,
}

impl Scraper {
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

    // ===============================================

    pub async fn run(self) -> Result<()> {
        let episodes = self.scan_unscraped_episodes().await?;
        self.scrape(episodes).await?;
        Ok(())
    }


    // ===============================================

    async fn scan_unscraped_episodes(&self) -> Result<Vec<Episode>> {
        let out = std::sync::Arc::new(parking_lot::Mutex::new(Some(Vec::<Episode>::new())));
        let mut handles = Vec::<tokio::task::JoinHandle<Result<(), anyhow::Error>>>::with_capacity(
            self.series_vec.len(),
        );
        for &series in self.series_vec.iter() {
            let out = std::sync::Arc::clone(&out);
            let json_rw = self.rw_json.arc_clone();
            let h = tokio::spawn(async move {
                let scraped = json_rw.read_episodes(&series).unwrap();
                let all = Scraper::scrape_episode(series).await?;

                let mut to_scraped = all
                    .into_iter()
                    .filter(|episode| !scraped.iter().any(|scraped| scraped.id() == episode.id()))
                    .collect::<Vec<Episode>>();

                let mut out = out.lock();
                let out = out.as_mut().unwrap();
                out.append(&mut to_scraped);

                Ok::<(), anyhow::Error>(())
            });
            handles.push(h);
        }

        for h in handles {
            h.await.unwrap()?;
        }

        let mut out = out.lock();
        Ok(out.take().unwrap())
    }

    // ===============================================

    async fn scrape_download_url(episode: &mut Episode) -> Result<()> {
        let config = chromiumoxide::BrowserConfig::builder()
            // .with_head()
            .build()
            .map_err(|e| anyhow!(e))?;
        let (browser, mut handler) = chromiumoxide::Browser::launch(config)
            .await
            .map_err(|e| anyhow!(e))?;

        let addr = browser.websocket_address();
        let port = &addr[15..addr.rfind("/devtools").unwrap()];
        println!(
            "port {} scraping {:?} {}",
            port,
            episode.series(),
            episode.url()
        );

        let close_handler = std::sync::Arc::new(parking_lot::Mutex::new(false));
        let close_handler_clone = std::sync::Arc::clone(&close_handler);
        let handle = async_std::task::spawn(async move {
            loop {
                if *close_handler_clone.lock() == true {
                    break;
                }
                let _event = handler.next().await.unwrap();
            }
        });

        let page = browser.new_page(episode.url()).await?;
        let download_url = page
            .find_element(".download>a")
            .await?
            .attribute("href")
            .await?
            .unwrap();
        let download_url = &download_url[..download_url.rfind(".mp3").unwrap() + 4];

        episode.set_download_url(download_url.to_string());

        drop(browser);
        *close_handler.lock() = true;
        handle.await;

        Ok(())
    }

    // ===============================================

    pub async fn scrape(&self, episodes: Vec<Episode>) -> Result<()> {
        // TODO parking_lot::Mutex or tokio::sync::Mutex?????
        let episodes = std::sync::Arc::new(tokio::sync::Mutex::new(episodes));
        let mut handles_out = Vec::<tokio::task::JoinHandle<()>>::with_capacity(self.max_worker);

        for _ in 0..self.max_worker {
            let episodes = std::sync::Arc::clone(&episodes);
            // let episode_rw_map = std::sync::Arc::clone(&self.episode_rw_map);
            let json_rw = self.rw_json.arc_clone();
            let h = tokio::spawn(async move {
                loop {
                    let mut episodes = episodes.lock().await;
                    let episode = episodes.pop();
                    drop(episodes); // drop the guard immediately so that other thread can have it.
                    if let Some(mut episode) = episode {
                        while let Err(err) = Scraper::scrape_download_url(&mut episode).await {
                            println!("SCRAPE FAIL: {:?}: {:?}", err, episode.url());
                        }

                        json_rw.write_episode(episode).unwrap();
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

    // ===============================================

    async fn scrape_episode(series: Series) -> Result<Vec<Episode>> {
        let config = chromiumoxide::BrowserConfig::builder()
            // .with_head()
            .build()
            .map_err(|e| anyhow!(e))?;
        let (browser, mut handler) = chromiumoxide::Browser::launch(config)
            .await
            .map_err(|e| anyhow!(e))?;

        let close_handler = std::sync::Arc::new(parking_lot::Mutex::new(false));
        let close_handler_clone = std::sync::Arc::clone(&close_handler);
        let handle = async_std::task::spawn(async move {
            loop {
                if *close_handler_clone.lock() == true {
                    break;
                }
                let _event = handler.next().await.unwrap();
            }
        });

        let page = browser.new_page(series.url()).await?;
        let elems = page.find_elements(".archive_entry").await?;
        let mut out = Vec::<Episode>::with_capacity(elems.len());

        for elem in elems {
            let number = if let Ok(a) = elem.find_element(".episode_number a").await {
                let number_raw = a.inner_text().await.unwrap().unwrap();
                let number_str = number_raw.trim_start_matches("NO. ");
                // number_str.parse::<usize>().unwrap()
                number_str.to_string()
            } else {
                continue;
            };

            let (title, url) = if let Ok(a) = elem.find_element(".entry_content a").await {
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

            let episode = Episode::new(series, number, title, url, date);
            out.push(episode);
        }

        drop(browser);
        *close_handler.lock() = true;
        handle.await;

        Ok(out)
    }

    // ===============================================
}

// ===============================================================================================
