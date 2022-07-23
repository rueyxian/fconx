use serde::Deserialize;
use serde::Serialize;

use crate::config::Series;

///
#[derive(Debug,Clone, Serialize, Deserialize)]
pub struct Episode {
    id: String,
    sha1: Option<String>,
    series: Series,
    number: String,
    title: String,
    #[serde(with = "string_serde_datetime")]
    date: chrono::DateTime<chrono::Utc>,
    page_url: String,
    download_url: Option<String>,
}

///
impl Episode {
    ///
    pub fn new(
        series: Series,
        number: String,
        title: String,
        date: chrono::DateTime<chrono::Utc>,
        page_url: String,
    ) -> Episode {
        let id = uuid::Uuid::new_v5(&uuid::Uuid::NAMESPACE_DNS, page_url.to_string().as_bytes())
            .to_string();
        Episode {
            id,
            sha1: None,
            series,
            number,
            title,
            page_url,
            date,
            download_url: None,
        }
    }

    ///
    pub fn id(&self) -> &str {
        self.id.as_str()
    }

    ///
    pub fn sha1(&self) -> Option<String> {
        self.sha1.to_owned()
    }

    ///
    pub fn number(&self) -> &str {
        self.number.as_str()
    }

    ///
    pub fn title(&self) -> &str {
        &self.title
    }

    ///
    pub fn page_url(&self) -> &str {
        &self.page_url
    }

    ///
    pub fn download_url(&self) -> Option<String> {
        self.download_url.to_owned()
    }

    ///
    pub fn series(&self) -> Series {
        self.series
    }

    ///
    pub fn set_sha1(&mut self, sha1: String) {
        self.sha1 = Some(sha1);
    }

    ///
    pub fn set_download_url(&mut self, download_url: String) {
        self.download_url = Some(download_url);
    }
}

mod string_serde_datetime {

    ///
    pub fn serialize<S>(
        date: &chrono::DateTime<chrono::Utc>,
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(date.to_string().as_str())
    }

    ///
    pub(crate) fn deserialize<'de, D>(
        deserializer: D,
    ) -> Result<chrono::DateTime<chrono::Utc>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use std::str::FromStr;
        let dt_str: String = serde::Deserialize::deserialize(deserializer)?;
        chrono::DateTime::<chrono::Utc>::from_str(dt_str.as_str())
            .map_err(|_| serde::de::Error::custom("chrono::DateTime parsing error"))
    }
}
