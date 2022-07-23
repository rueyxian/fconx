use crate::canceller::Canceller;
use crate::config::Series;
use crate::episode::Episode;
use crate::hasher::Sha1Hasher;
use crate::logger::Logger;
use crate::rw::RWJson;
use crate::rw::RWMp3;

///
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

///
pub(crate) struct Downloader {
    logger: std::sync::Arc<Logger>,
    canceller: std::sync::Arc<Canceller>,
    thread_max: usize,
    series_vec: std::sync::Arc<Vec<Series>>,
    rw_json: std::sync::Arc<RWJson>,
    rw_mp3: std::sync::Arc<RWMp3>,
}

///
impl Downloader {
    ///
    pub(crate) fn new(
        logger: std::sync::Arc<Logger>,
        canceller: std::sync::Arc<Canceller>,
        thread_max: usize,
        series_vec: std::sync::Arc<Vec<Series>>,
        rw_json: std::sync::Arc<RWJson>,
        rw_mp3: std::sync::Arc<RWMp3>,
    ) -> Downloader {
        Downloader {
            canceller,
            logger,
            thread_max,
            series_vec,
            rw_json,
            rw_mp3,
        }
    }

    ///
    pub(crate) async fn run(&self) -> Result<()> {
        // let to_download = self.available_episodes().await?;
        let to_download = self.scan_not_downloaded_mp3s().await?;
        self.logger.log_to_download(&to_download).await;

        self.download_mp3s(to_download).await?;
        Ok(())
    }

    // ///
    // /// unused
    // async fn available_episodes(&self) -> Result<Vec<Episode>> {
    //     let out_mutex = {
    //         let v = Some(Vec::<Episode>::new());
    //         std::sync::Arc::new(parking_lot::Mutex::new(v))
    //     };
    //     let mut handles = Vec::<tokio::task::JoinHandle<()>>::with_capacity(self.series_vec.len());
    //     for &series in self.series_vec.iter() {
    //         let out_mutex = std::sync::Arc::clone(&out_mutex);
    //         let rw_json = self.rw_json.arc_clone();
    //         let h = tokio::spawn(async move {
    //             let mut no_sha1 = rw_json.read_episodes_no_sha1(&series).unwrap();
    //             {
    //                 let mut out_guard = out_mutex.lock();
    //                 let out = out_guard.as_mut().unwrap();
    //                 out.append(&mut no_sha1);
    //             }
    //         });
    //         handles.push(h);
    //         //
    //     }
    //     for h in handles {
    //         h.await.unwrap();
    //     }
    //     let out = {
    //         let mut guard = out_mutex.lock();
    //         guard.take().unwrap()
    //     };
    //     Ok(out)
    // }

    ///
    async fn scan_not_downloaded_mp3s(&self) -> Result<Vec<Episode>> {
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
    async fn download_mp3s(&self, episodes: Vec<Episode>) -> Result<()> {
        // TODO: Refector the code:
        // break it down to Job Struct and Worker struct.
        let episodes_count = episodes.len();
        let episodes_mutex = std::sync::Arc::new(tokio::sync::Mutex::new(episodes));
        let mut handles = Vec::<tokio::task::JoinHandle<()>>::with_capacity(self.thread_max);
        for idx in 0..usize::min(self.thread_max, episodes_count) {
            let logger = self.logger.arc_clone();
            let canceller = self.canceller.arc_clone();
            let episodes_mutex = std::sync::Arc::clone(&episodes_mutex);
            let rw_mp3 = self.rw_mp3.arc_clone();
            let rw_json = self.rw_json.arc_clone();
            let h = tokio::spawn(async move {
                let mut hasher = Sha1Hasher::new();
                loop {
                    if canceller.is_cancel() {
                        break;
                    }
                    let mut episode = {
                        let mut episodes_guard = episodes_mutex.lock().await;
                        let episode = episodes_guard.pop();
                        drop(episodes_guard);
                        match episode {
                            Some(episode) => episode,
                            None => break,
                        }
                    };

                    logger.log_download_start(idx, &episode).await;
                    let bytes = match Downloader::download_mp3(&episode).await {
                        Ok(b) => b,
                        Err(err) => {
                            // TODO log err
                            logger.log_download_error(idx, &episode).await;
                            continue;
                        }
                    };
                    let sha1 = hasher.create_sha1(&bytes);
                    episode.set_sha1(sha1);
                    rw_mp3.write_mp3(&episode, bytes).await.unwrap();
                    rw_json.edit_episode(&episode).unwrap();

                    logger.log_download_done(idx, &episode).await;
                }
                logger.log_download_thread_kill(idx).await;
            });
            handles.push(h);
        }
        for h in handles {
            h.await.unwrap();
        }
        Ok(())
    }

    ///
    async fn download_mp3(episode: &Episode) -> Result<bytes::Bytes> {
        let download_url = episode.download_url().unwrap();
        let bytes = {
            let response = reqwest::get(download_url).await.unwrap();
            response.bytes().await.unwrap()
        };
        Ok(bytes)
    }
}
