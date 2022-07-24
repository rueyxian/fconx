// use crate::;

use crate::config::Series;
use crate::episode::Episode;
use crate::logger::Log;

///
#[derive(Debug)]
pub(crate) struct Logger {
    log_send: flume::Sender<Log>,
}

///
impl Logger {
    ///
    /// reference: https://stackoverflow.com/questions/69462729/how-can-i-get-an-equivalent-to-sync-channel0-in-tokio
    pub(crate) fn new() -> (std::sync::Arc<Logger>, flume::Receiver<Log>) {
        // let (log_send, log_recv) = tokio::sync::mpsc::channel::<Log>(1);
        let (log_send, log_recv) = flume::unbounded();

        let logger = Logger { log_send };

        (std::sync::Arc::new(logger), log_recv)
    }

    ///
    pub(crate) fn arc_clone(self: &std::sync::Arc<Logger>) -> std::sync::Arc<Logger> {
        std::sync::Arc::clone(self)
    }

    ///
    pub(crate) async fn log(self: &std::sync::Arc<Logger>, log: Log) {
        self.log_send.send(log).unwrap();
    }

    // ========================

    ///
    pub(crate) async fn log_scrape_episode_start(
        self: &std::sync::Arc<Logger>,
        idx: usize,
        series: Series,
    ) {
        self.log(Log::ScrapeEpisodesStart { idx, series }).await;
    }

    ///
    pub(crate) async fn log_scrape_episode_error(
        self: &std::sync::Arc<Logger>,
        idx: usize,
        series: Series,
    ) {
        self.log(Log::ScrapeEpisodesError { idx, series }).await;
    }

    ///
    pub(crate) async fn log_scrape_episode_thread_kill(self: &std::sync::Arc<Logger>, idx: usize) {
        self.log(Log::ScrapeEpisodesThreadKill { idx }).await;
    }

    // ========================

    ///
    pub(crate) async fn log_new_episodes(
        self: &std::sync::Arc<Logger>,
        series: Series,
        episodes: &Vec<Episode>,
    ) {
        self.log(Log::NewEpisodes {
            series,
            episodes: episodes.clone(),
        })
        .await;
    }

    // ========================

    ///
    pub(crate) async fn log_to_scrape(self: &std::sync::Arc<Logger>, episodes: &Vec<Episode>) {
        self.log(Log::ToScrape {
            episodes: episodes.clone(),
        })
        .await;
    }

    ///
    pub(crate) async fn log_scrape_download_url_start(
        self: &std::sync::Arc<Logger>,
        idx: usize,
        episode: &Episode,
    ) {
        self.log(Log::ScrapeDownloadUrlStart {
            idx,
            episode: episode.clone(),
        })
        .await;
    }

    ///
    pub(crate) async fn log_scrape_download_url_done(
        self: &std::sync::Arc<Logger>,
        idx: usize,
        episode: &Episode,
    ) {
        self.log(Log::ScrapeDownloadUrlDone {
            idx,
            episode: episode.clone(),
        })
        .await;
    }

    ///
    pub(crate) async fn log_scrape_download_url_error(
        self: &std::sync::Arc<Logger>,
        idx: usize,
        episode: &Episode,
    ) {
        self.log(Log::ScrapeDownloadUrlError {
            idx,
            episode: episode.clone(),
        })
        .await;
    }

    ///
    pub(crate) async fn log_scrape_download_url_thread_kill(
        self: &std::sync::Arc<Logger>,
        idx: usize,
    ) {
        self.log(Log::ScrapeDownloadUrlThreadKill { idx }).await;
    }

    // ========================

    ///
    pub(crate) async fn log_existing_mp3_found(
        self: &std::sync::Arc<Logger>,
        idx: usize,
        series: Series,
        file_path: std::path::PathBuf,
    ) {
        self.log(Log::ExistingMp3Found {
            idx,
            series,
            file_path,
        })
        .await;
    }

    // ========================

    ///
    pub(crate) async fn log_to_download(self: &std::sync::Arc<Logger>, episodes: &Vec<Episode>) {
        self.log(Log::ToDownload {
            episodes: episodes.clone(),
        })
        .await;
    }

    ///
    pub(crate) async fn log_download_start(
        self: &std::sync::Arc<Logger>,
        idx: usize,
        episode: &Episode,
    ) {
        self.log(Log::DownloadStart {
            idx,
            episode: episode.clone(),
        })
        .await;
    }

    ///
    pub(crate) async fn log_download_done(
        self: &std::sync::Arc<Logger>,
        idx: usize,
        episode: &Episode,
    ) {
        self.log(Log::DownloadDone {
            idx,
            episode: episode.clone(),
        })
        .await;
    }

    ///
    pub(crate) async fn log_download_error(
        self: &std::sync::Arc<Logger>,
        idx: usize,
        episode: &Episode,
    ) {
        self.log(Log::DownloadError {
            idx,
            episode: episode.clone(),
        })
        .await;
    }

    ///
    pub(crate) async fn log_download_thread_kill(self: &std::sync::Arc<Logger>, idx: usize) {
        self.log(Log::DownloadThreadKill { idx }).await;
    }
}
