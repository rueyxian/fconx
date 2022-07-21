use crate::config::Config;
use crate::downloader::Downloader;
use crate::rw::RWJson;
use crate::rw::RWMp3;
use crate::scraper::DownloadUrlScraper;
use crate::scraper::EpisodeScraper;

///
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

///
pub struct Fconx {
    //
    // rw_json: std::sync::Arc<RWJson>,
    // rw_mp3: std::sync::Arc<RWMp3>,
    episode_scraper: EpisodeScraper,
    download_url_scraper: DownloadUrlScraper,
    downloader: Downloader,
}

impl Fconx {
    pub fn new() -> Fconx {
        let config = Config::new_arc();
        let rw_json = RWJson::new_arc(&config);
        let rw_mp3 = RWMp3::new_arc(&config, 64);
        let episode_scraper = EpisodeScraper::new(config.series_vec(), rw_json.arc_clone());
        let download_url_scraper =
            DownloadUrlScraper::new(32, config.series_vec(), rw_json.arc_clone());
        let downloader = Downloader::new(
            8,
            config.series_vec(),
            rw_json.arc_clone(),
            rw_mp3.arc_clone(),
        );
        Fconx {
            // rw_json,
            // rw_mp3,
            episode_scraper,
            download_url_scraper,
            downloader,
        }
    }

    pub async fn scrape_episodes(&self) -> Result<()> {
        self.episode_scraper.run().await?;
        Ok(())
    }

    pub async fn scrape_download_url(&self) -> Result<()> {
        self.download_url_scraper.run().await?;
        Ok(())
    }

    pub async fn download_mp3(&self) -> Result<()> {
        self.downloader.run().await?;
        Ok(())
    }
}
