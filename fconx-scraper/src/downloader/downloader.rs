// ========================
// ===============================================
// ===============================================================================================

use crate::episode::Episode;
use crate::rw::RWJson;
use crate::config::Series;

// ===============================================================================================

pub struct Downloader {
    max_worker: usize,
    series_vec: Vec<Series>,
    rw_json: std::sync::Arc<RWJson>,
    // episode_rw_map:
    //     std::sync::Arc<std::collections::HashMap<Series, parking_lot::Mutex<EpisodeRW>>>,
}

impl Downloader {
    pub fn new(
        max_worker: usize,
        series_vec: Vec<Series>,
        rw_json: std::sync::Arc<RWJson>,
    ) -> Downloader {
        Downloader {
            max_worker,
            series_vec,
            rw_json,
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let episode_vec = self.scan_undownloaded_episodes()?;
        self.download(episode_vec).await?;

        Ok(())
    }

    fn scan_undownloaded_episodes(&self) -> anyhow::Result<Vec<Episode>> {
        let mut out = Vec::<Episode>::new();
        for &series in self.series_vec.iter() {
            let all = self.rw_json.read_episodes(&series)?;
            // let downloaded_episodes = all_episodes;
            let mut undownloaded = all;
            // TODO filter downloaded episodes
            out.append(&mut undownloaded);
        }
        Ok(out)
    }

    pub async fn download(&self, episodes: Vec<Episode>) -> anyhow::Result<()> {
        let episodes = std::sync::Arc::new(tokio::sync::Mutex::new(episodes));
        let mut handles = Vec::<tokio::task::JoinHandle<()>>::with_capacity(self.max_worker);
        for _ in 0..self.max_worker {
            //
            let episodes = std::sync::Arc::clone(&episodes);
            let h = tokio::spawn(async move {
                loop {
                    //
                    let mut episodes = episodes.lock().await;
                    let episode = episodes.pop();
                    drop(episodes);

                    if let Some(mut episode) = episode {
                        let download_url = episode.download_url().unwrap();

                        // while let Err(err) = Downloader::download_episode(&mut episode).await {
                        //     println!("SCRAPE FAIL: {:?}: {:?}", err, episode.url());
                        // }
                    } else {
                        break;
                    }

                    // drop(episodes);
                }
            });
            handles.push(h);
        }

        for h in handles {
            h.await.unwrap();
        }

        Ok(())
    }

    async fn download_episode(episode: &Episode) -> anyhow::Result<()> {
        //
        Ok(())
    }
}

// ===============================================================================================

// async fn download(&self, episode: Episode>) -> Result<()>{
//
// }
