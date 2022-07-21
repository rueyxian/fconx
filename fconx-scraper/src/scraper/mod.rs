// mod scraper;

mod episode_scraper;

mod download_url_scraper;

// pub use crate::scraper::scraper::Scraper;

pub(crate) use crate::scraper::download_url_scraper::DownloadUrlScraper;
pub(crate) use crate::scraper::episode_scraper::EpisodeScraper;
