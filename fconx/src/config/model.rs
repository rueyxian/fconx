use serde::Deserialize;
use serde::Serialize;

use crate::config::Series;

///
#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ConfigModel {
    #[serde(with = "string_serde_path_buf")]
    pub dir_path: std::path::PathBuf,
    #[serde(with = "vec_string_serde_vec_series")]
    pub series_vec: Vec<Series>,
}

///
impl Default for ConfigModel {
    fn default() -> Self {
        let dir_path = {
            let homedir = std::env::var("HOME").unwrap();
            let dir_path = std::path::Path::new(&homedir);
            dir_path.join("Music/fconx/")
        };

        let series_vec = vec![
            // "FR".to_string(),
            // "NSQ".to_string(),
            // "FMD".to_string(),
            // "PIMA".to_string(),
            // "OL".to_string(),
            Series::FR,
            Series::NSQ,
            Series::FMD,
            Series::PIMA,
            Series::OL,
        ];

        Self {
            dir_path,
            series_vec,
        }
    }
}

///
mod vec_string_serde_vec_series {
    use crate::config::Series;

    ///
    pub fn serialize<S>(series_vec: &Vec<Series>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let str_vec = series_vec
            .iter()
            .map(|series| {
                match series {
                    Series::FR => "FR".to_string(),
                    Series::NSQ => "NSQ".to_string(),
                    Series::FMD => "FMD".to_string(),
                    Series::PIMA => "PIMA".to_string(),
                    Series::OL => "OL".to_string(),
                }
            })
            .collect::<Vec<String>>();
        serde::Serialize::serialize(&str_vec, serializer)
    }

    ///
    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<Vec<Series>, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str_vec: Vec<String> = serde::Deserialize::deserialize(deserializer)?;
        let mut series_vec = Vec::with_capacity(str_vec.len());
        for s in str_vec {
            let series = Series::try_from(s).map_err(|_| {
                serde::de::Error::custom("expect one of these: FR, NSQ, FMD, PIMA, OL")
            })?;
            series_vec.push(series);
        }
        Ok(series_vec)
    }
}

///
mod string_serde_path_buf {
    ///
    pub fn serialize<S>(path_buf: &std::path::PathBuf, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = path_buf.to_string_lossy().to_string();
        serde::Serialize::serialize(&s, serializer)
    }

    ///
    pub(crate) fn deserialize<'de, D>(deserializer: D) -> Result<std::path::PathBuf, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: String = serde::Deserialize::deserialize(deserializer)?;
        Ok(std::path::PathBuf::new().join(s))
    }
}
