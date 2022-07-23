use crate::canceller::Canceller;
use crate::config::Config;
use crate::downloader::Downloader;
use crate::logger::Log;
use crate::logger::Logger;
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
    // logger: std::sync::Arc<Logger>,
    canceller: std::sync::Arc<Canceller>,
    episode_scraper: EpisodeScraper,
    download_url_scraper: DownloadUrlScraper,
    downloader: Downloader,
}

impl Fconx {
    pub fn new() -> (Fconx, flume::Receiver<Log>, std::sync::Arc<Canceller>) {
        let (logger, log_recv) = Logger::new();

        let canceller = Canceller::new();

        let config = Config::new_arc();
        let rw_json = RWJson::new_arc(&config);
        let rw_mp3 = RWMp3::new_arc(logger.arc_clone(), canceller.arc_clone(), &config, 64);

        let episode_scraper =
            EpisodeScraper::new(logger.arc_clone(), config.series_vec(), rw_json.arc_clone());

        let download_url_scraper = DownloadUrlScraper::new(
            logger.arc_clone(),
            canceller.arc_clone(),
            num_cpus::get(),
            config.series_vec(),
            rw_json.arc_clone(),
        );

        let downloader = Downloader::new(
            logger.arc_clone(),
            canceller.arc_clone(),
            8,
            config.series_vec(),
            rw_json.arc_clone(),
            rw_mp3.arc_clone(),
        );

        let fconx = Fconx {
            // rw_json,
            // rw_mp3,
            // logger,
            canceller: canceller.arc_clone(),
            episode_scraper,
            download_url_scraper,
            downloader,
        };
        (fconx, log_recv, canceller)
    }

    pub async fn scrape_episodes(&self) -> Result<()> {
        if self.canceller.is_cancel() {
            return Ok(());
        }
        self.episode_scraper.run().await?;
        Ok(())
    }

    pub async fn scrape_download_url(&self) -> Result<()> {
        if self.canceller.is_cancel() {
            return Ok(());
        }
        self.download_url_scraper.run().await?;
        Ok(())
    }

    pub async fn download_mp3(&self) -> Result<()> {
        if self.canceller.is_cancel() {
            return Ok(());
        }
        self.downloader.run().await?;
        Ok(())
    }
}
