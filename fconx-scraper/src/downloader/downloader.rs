use crate::config::Series;
use crate::episode::Episode;
use crate::hasher::Sha1Hasher;
use crate::rw::RWJson;
use crate::rw::RWMp3;

///
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

///
pub(crate) struct Downloader {
    max_worker: usize,
    series_vec: std::sync::Arc<Vec<Series>>,
    rw_json: std::sync::Arc<RWJson>,
    rw_mp3: std::sync::Arc<RWMp3>,
}

///
impl Downloader {
    ///
    pub(crate) fn new(
        max_worker: usize,
        series_vec: std::sync::Arc<Vec<Series>>,
        rw_json: std::sync::Arc<RWJson>,
        rw_mp3: std::sync::Arc<RWMp3>,
    ) -> Downloader {
        Downloader {
            max_worker,
            series_vec,
            rw_json,
            rw_mp3,
        }
    }

    ///
    pub(crate) async fn run(&self) -> Result<()> {
        println!("================ find downloaded episodes ================");
        // let to_download = self.available_episodes().await?;
        let to_download = self.scan_not_downloaded_episodes().await?;
        println!("{} episodes are available to download", to_download.len());
        println!("================ start downloading  ================");
        self.download_episodes(to_download).await?;
        println!("================ done downloading  ================");
        Ok(())
    }

    ///
    async fn available_episodes(&self) -> Result<Vec<Episode>> {
        let out_mutex = {
            let v = Some(Vec::<Episode>::new());
            std::sync::Arc::new(parking_lot::Mutex::new(v))
        };
        let mut handles = Vec::<tokio::task::JoinHandle<()>>::with_capacity(self.series_vec.len());
        for &series in self.series_vec.iter() {
            let out_mutex = std::sync::Arc::clone(&out_mutex);
            let rw_json = self.rw_json.arc_clone();

            let h = tokio::spawn(async move {
                let mut no_sha1 = rw_json.read_episodes_no_sha1(&series).unwrap();
                {
                    let mut out_guard = out_mutex.lock();
                    let out = out_guard.as_mut().unwrap();
                    out.append(&mut no_sha1);
                }
            });
            handles.push(h);
            //
        }
        for h in handles {
            h.await.unwrap();
        }

        let out = {
            let mut guard = out_mutex.lock();
            guard.take().unwrap()
        };
        Ok(out)
    }

    ///
    async fn scan_not_downloaded_episodes(&self) -> Result<Vec<Episode>> {
        let out_mutex = {
            let v = Some(Vec::<Episode>::new());
            std::sync::Arc::new(parking_lot::Mutex::new(v))
        };
        let mut handles = Vec::<tokio::task::JoinHandle<()>>::with_capacity(self.series_vec.len());
        for &series in self.series_vec.iter() {
            let out_mutex = std::sync::Arc::clone(&out_mutex);
            let rw_mp3 = self.rw_mp3.arc_clone();
            let rw_json = self.rw_json.arc_clone();
            let h = tokio::spawn(async move {
                let all = rw_json.read_all_episodes(&series).unwrap();
                let downloaded_sha1s = rw_mp3.read_mp3s_and_to_sha1(series).await.unwrap();
                let mut not_downloaded = all
                    .into_iter()
                    .filter(|ep| {
                        if let Some(sha1) = ep.sha1() {
                            !downloaded_sha1s.contains(&sha1)
                        } else {
                            true
                        }
                    })
                    .collect::<Vec<Episode>>();
                {
                    let mut out_guard = out_mutex.lock();
                    let out = out_guard.as_mut().unwrap();
                    out.append(&mut not_downloaded);
                }
            });
            handles.push(h);
        }

        for h in handles {
            h.await.unwrap();
        }

        let out = {
            let mut guard = out_mutex.lock();
            guard.take().unwrap()
        };
        Ok(out)
    }

    ///
    async fn download_episodes(&self, episodes: Vec<Episode>) -> Result<()> {
        // TODO: Refector the code:
        // break it down to Job Struct and Worker struct.
        let episodes_count = episodes.len();
        let episodes_mutex = std::sync::Arc::new(tokio::sync::Mutex::new(episodes));
        let mut handles = Vec::<tokio::task::JoinHandle<()>>::with_capacity(self.max_worker);
        for _ in 0..usize::min(self.max_worker, episodes_count) {
            let episodes_mutex = std::sync::Arc::clone(&episodes_mutex);
            let rw_mp3 = self.rw_mp3.arc_clone();
            let rw_json = self.rw_json.arc_clone();
            let h = tokio::spawn(async move {
                let mut hasher = Sha1Hasher::new();
                loop {
                    let episode = {
                        episodes_mutex.lock().await.pop() // drop the guard immediately
                    };
                    if let Some(mut episode) = episode {
                        let bytes = match Downloader::download_episode(&episode).await {
                            Ok(b) => b,
                            Err(err) => {
                                println!(
                                    "DOWNLOAD ERROR: {:?} {} {:?}: {:?}",
                                    episode.series(),
                                    episode.number(),
                                    episode.title(),
                                    err,
                                );
                                continue;
                            }
                        };
                        let sha1 = hasher.create_sha1(&bytes);
                        episode.set_sha1(sha1);
                        rw_mp3.write_mp3(&episode, bytes).await.unwrap();
                        rw_json.edit_episode(episode).unwrap();
                    } else {
                        break;
                    }
                }
            });
            handles.push(h);
        }
        for h in handles {
            h.await.unwrap();
        }
        Ok(())
    }

    ///
    async fn download_episode(episode: &Episode) -> Result<bytes::Bytes> {
        println!(
            "downloading: {:?} {} - {}",
            episode.series(),
            episode.number(),
            episode.title()
        );
        let download_url = episode.download_url().unwrap();
        let bytes = {
            let response = reqwest::get(download_url).await.unwrap();
            response.bytes().await.unwrap()
        };
        Ok(bytes)
    }
}
