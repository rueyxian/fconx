use serde::Deserialize;
use serde::Serialize;
use url::Url;

#[derive(Debug)]
pub struct ParseSeriesError(String);

///
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, Serialize, Deserialize)]
pub enum Series {
    FR,
    NSQ,
    FMD,
    PIMA,
    OL,
}

///
impl Series {
    const URL_FR: &'static str = "https://freakonomics.com/series-full/freakonomics-radio/";
    const URL_NSQ: &'static str = "https://freakonomics.com/series-full/nsq/";
    const URL_FMD: &'static str = "https://freakonomics.com/series-full/bapu/";
    const URL_PIMA: &'static str = "https://freakonomics.com/series-full/people-i-mostly-admire/";
    const URL_OL: &'static str = "https://freakonomics.com/series-full/off-leash/";
    // const URL_SBTI: &'static str =
    //     "https://freakonomics.com/series-full/sudhir-breaks-the-internet/";

    pub fn url(&self) -> Url {
        match self {
            Series::FR => Url::parse(Series::URL_FR).unwrap(),
            Series::NSQ => Url::parse(Series::URL_NSQ).unwrap(),
            Series::FMD => Url::parse(Series::URL_FMD).unwrap(),
            Series::PIMA => Url::parse(Series::URL_PIMA).unwrap(),
            Series::OL => Url::parse(Series::URL_OL).unwrap(),
        }
    }

    pub fn data_json_filename(&self) -> String {
        match self {
            Series::FR => "fr.json".to_string(),
            Series::NSQ => "nsq.json".to_string(),
            Series::FMD => "fmd.json".to_string(),
            Series::PIMA => "pima.json".to_string(),
            Series::OL => "ol.json".to_string(),
        }
    }

    pub fn mp3_dirname(&self) -> String {
        match self {
            Series::FR => "Freakonomics Radio".to_string(),
            Series::NSQ => "No Stupid Question".to_string(),
            Series::FMD => "Freakonomics MD".to_string(),
            Series::PIMA => "People I Mostly Admire".to_string(),
            Series::OL => "Off Leash".to_string(),
        }
    }
}

///
impl TryFrom<String> for Series {
    type Error = ParseSeriesError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        match value.as_str() {
            "FR" => Ok(Series::FR),
            "NSQ" => Ok(Series::NSQ),
            "FMD" => Ok(Series::FMD),
            "PIMA" => Ok(Series::PIMA),
            "OL" => Ok(Series::OL),
            _ => Err(ParseSeriesError(value)),
        }
    }
}
