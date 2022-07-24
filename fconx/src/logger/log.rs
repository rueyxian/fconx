use crate::config::Series;
use crate::episode::Episode;

///
#[derive(Debug)]
pub enum Log {
    ///
    ScrapeEpisodesStart { idx: usize, series: Series },

    ///
    ScrapeEpisodesThreadKill { idx: usize },

    ///
    ScrapeEpisodesError { idx: usize, series: Series },

    // ========================
    ///
    NewEpisodes {
        series: Series,
        episodes: Vec<Episode>,
    },

    // ========================
    ///
    ToScrape { episodes: Vec<Episode> },

    ///
    ScrapeDownloadUrlStart { idx: usize, episode: Episode },

    ///
    ScrapeDownloadUrlDone { idx: usize, episode: Episode },

    ///
    ScrapeDownloadUrlError { idx: usize, episode: Episode },

    ///
    ScrapeDownloadUrlThreadKill { idx: usize },

    // ========================
    ///
    ExistingMp3Found {
        idx: usize,
        series: Series,
        file_path: std::path::PathBuf,
    },

    // ========================
    ///
    ToDownload { episodes: Vec<Episode> },

    ///
    DownloadStart { idx: usize, episode: Episode },

    ///
    DownloadDone { idx: usize, episode: Episode },

    ///
    DownloadError { idx: usize, episode: Episode },

    ///
    DownloadThreadKill { idx: usize },
}
