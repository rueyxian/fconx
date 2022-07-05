// ========================
// ===============================================
// ===============================================================================================

use chrono::NaiveDate;
use serde::Deserialize;
use serde::Serialize;
use uuid::Uuid;

use crate::config::Series;

// ===============================================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct Episode {
    id: String,
    series: Series,
    number: String,
    title: String,
    url: String,
    date: String,
    download_url: Option<String>,
    // downloaded: bool,
    binary_sha1: Option<String>,
}

impl Episode {
    pub fn new(
        series: Series,
        number: String,
        title: String,
        url: String,
        date: NaiveDate,
    ) -> Episode {
        let id = Uuid::new_v5(&Uuid::NAMESPACE_DNS, url.to_string().as_bytes()).to_string();
        Episode {
            id,
            series,
            number,
            title,
            url,
            date: date.to_string(),
            download_url: None,
            // downloaded: false,
            binary_sha1: None,
        }
    }

    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn download_url(&self) -> Option<&str> {
        Some(self.url.as_str())
    }

    pub fn series(&self) -> Series {
        self.series
    }

    pub fn set_download_url(&mut self, download_url: String) {
        self.download_url = Some(download_url)
    }
}

// ===============================================================================================
