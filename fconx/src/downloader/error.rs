use crate::episode::Episode;

///
#[derive(Debug)]
pub enum DownloadError {
    MissingDownloadUrl(Episode),
    GetRequest(String),
}

impl std::fmt::Display for DownloadError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // write!(f, "DownloadError: MissingDownloadUrl {}", )
        match self {
            DownloadError::MissingDownloadUrl(episode) => {
                write!(
                    f,
                    "DownloadError::MissingDownloadUrl: {:?} {} {}",
                    episode.series(),
                    episode.number(),
                    episode.title()
                )
            }
            DownloadError::GetRequest(download_url) => write!(f, "DownloadError::GetRequest: {:?}", download_url),
        }
    }
}

impl std::error::Error for DownloadError {}
