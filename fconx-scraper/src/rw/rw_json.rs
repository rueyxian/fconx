use crate::config::Config;
use crate::config::Series;
use crate::episode::Episode;

///
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

///
pub struct RWJson {
    file_path_map: std::collections::HashMap<Series, parking_lot::Mutex<FilePath>>,
}

///
impl RWJson {
    ///
    pub fn new_arc(config: &std::rc::Rc<Config>) -> std::sync::Arc<RWJson> {
        let mut file_path_map = std::collections::HashMap::with_capacity(config.series_vec().len());
        for &series in config.series_vec().iter() {
            let file_path_mutex = {
                let file_path = FilePath::new(series, config.data_dir_path().as_path());
                parking_lot::Mutex::new(file_path)
            };
            file_path_map.insert(series, file_path_mutex);
        }
        std::sync::Arc::new(RWJson { file_path_map })
    }

    ///
    pub fn arc_clone(self: &std::sync::Arc<Self>) -> std::sync::Arc<RWJson> {
        std::sync::Arc::clone(&self)
    }


    ///
    fn overwrite_all_episode(
        self: &std::sync::Arc<Self>,
        series: &Series,
        episodes: Vec<Episode>,
    ) -> Result<()> {
        let file_path_mutex = self.file_path_map.get(series).unwrap();
        file_path_mutex
            .lock()
            .overwrite_all_episode(episodes)
            .unwrap();
        Ok(())
    }

    ///
    pub fn push_episode(self: &std::sync::Arc<Self>, episode: Episode) -> Result<()> {
        let file_path_mutex = self.file_path_map.get(&episode.series()).unwrap();
        file_path_mutex.lock().push_episode(episode).unwrap();
        Ok(())
    }

    ///
    pub fn read_all_episodes(self: &std::sync::Arc<Self>, series: &Series) -> Result<Vec<Episode>> {
        let file_path_mutex = self.file_path_map.get(series).unwrap();
        let episodes = file_path_mutex.lock().read_all_episode().unwrap();
        Ok(episodes)
    }

    ///
    pub fn edit_episode(self: &std::sync::Arc<Self>, episode: Episode) -> Result<()> {
        let series = episode.series();
        let all = self.read_all_episodes(&series).unwrap().into_iter();
        let mut filtered = all
            .filter(|ep| ep.id() != episode.id())
            .collect::<Vec<Episode>>();
        filtered.push(episode);
        self.overwrite_all_episode(&series, filtered).unwrap();
        Ok(())
    }
}

///
struct FilePath {
    file_path: std::path::PathBuf,
}

///
impl FilePath {
    ///
    fn new(series: Series, dir_path: &std::path::Path) -> FilePath {
        let file_path = dir_path.join(series.data_json_filename());
        FilePath { file_path }
    }

    ///
    fn path(&self) -> &std::path::Path {
        self.file_path.as_path()
    }

    ///
    fn overwrite_all_episode(&self, episodes: Vec<Episode>) -> Result<()> {
        use std::io::Write;
        let json_buf = serde_json::to_string_pretty(&episodes).unwrap();
        let mut file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(self.path())
            .unwrap();
        file.write(json_buf.as_bytes()).unwrap();
        Ok(())
    }

    ///
    fn push_episode(&self, episode: Episode) -> Result<()> {
        let mut episodes = self.read_all_episode().unwrap();
        episodes.push(episode);
        self.overwrite_all_episode(episodes).unwrap();
        Ok(())
    }

    ///
    fn read_all_episode(&self) -> Result<Vec<Episode>> {
        let file = std::fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(self.path())
            .unwrap();

        let mut reader = std::io::BufReader::new(file);
        let episodes: Vec<Episode> = serde_json::from_reader(&mut reader).unwrap_or(vec![]);
        Ok(episodes)
    }
}
