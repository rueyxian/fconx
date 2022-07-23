use crate::config::Series;
use crate::episode::Episode;
use crate::logger::Logger;
use crate::rw::RWJson;

///
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

///
pub(crate) struct EpisodeScraper {
    logger: std::sync::Arc<Logger>,
    series_vec: std::sync::Arc<Vec<Series>>,
    rw_json: std::sync::Arc<RWJson>,
}

///
impl EpisodeScraper {
    ///
    pub(crate) fn new(
        logger: std::sync::Arc<Logger>,
        series_vec: std::sync::Arc<Vec<Series>>,
        rw_json: std::sync::Arc<RWJson>,
    ) -> EpisodeScraper {
        EpisodeScraper {
            logger,
            series_vec,
            rw_json,
        }
    }


    ///
    pub(crate) async fn run(&self) -> Result<()> {
        self.scrape_and_write_episodes().await?;
        Ok(())
    }

    ///
    async fn scrape_and_write_episodes(&self) -> Result<()> {
        let mut handles =
            Vec::<tokio::task::JoinHandle<Result<()>>>::with_capacity(self.series_vec.len());
        for &series in self.series_vec.iter() {
            let logger = self.logger.arc_clone();
            let rw_json = self.rw_json.arc_clone();
            let h = tokio::spawn(async move {
                let mut from_json = rw_json.read_all_episodes(&series).unwrap();
                let mut new = {
                    let scraped = EpisodeScraper::scrape_episodes_by_series(series).await?;
                    scraped
                        .into_iter()
                        .filter(|episode| {
                            !from_json.iter().any(|scraped| scraped.id() == episode.id())
                        })
                        .collect::<Vec<Episode>>()
                };
                logger.log_new_episodes(series, &new).await;
                from_json.append(&mut new);
                rw_json.overwrite_all_episodes(&series, from_json)?;
                Ok::<(), Box<dyn std::error::Error + Send + Sync + 'static>>(())
            });
            handles.push(h);
        }
        for h in handles {
            h.await.unwrap()?;
        }
        Ok(())
    }

    ///
    async fn scrape_episodes_by_series(series: Series) -> Result<Vec<Episode>> {
        let browser = {
            let opts = headless_chrome::LaunchOptionsBuilder::default()
                // .headless(false)
                .idle_browser_timeout(std::time::Duration::from_millis(8_000))
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
                    use chrono::offset::TimeZone;
                    let dt_str = {
                        let d_str = elem_ref.text().next().unwrap();
                        format!("{} 00:00:00", d_str)
                    };
                    let naive = chrono::NaiveDateTime::parse_from_str(&dt_str, "%m/%d/%y %H:%M:%S")
                        .unwrap();
                    chrono::Utc.from_utc_datetime(&naive)
                } else {
                    continue;
                }
            };

            let episode = Episode::new(series, number, title, date, page_url);
            out.push(episode);
        }

        Ok(out)
    }
}
