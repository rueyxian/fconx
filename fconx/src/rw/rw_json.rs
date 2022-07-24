use crate::config::Config;
use crate::config::Series;
use crate::episode::Episode;

///
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

///
pub(crate) struct RWJson {
    rw_map: std::collections::HashMap<Series, parking_lot::Mutex<RW>>,
}

///
impl RWJson {
    ///
    pub(crate) fn new_arc(config: &std::sync::Arc<Config>) -> std::sync::Arc<RWJson> {
        let mut rw_map = std::collections::HashMap::with_capacity(config.series_vec().len());
        for &series in config.series_vec().iter() {
            let rw_mutex = {
                let file_path = RW::new(series, config.data_dir_path().as_path());
                parking_lot::Mutex::new(file_path)
            };
            rw_map.insert(series, rw_mutex);
        }
        std::sync::Arc::new(RWJson { rw_map })
    }

    ///
    pub(crate) fn arc_clone(self: &std::sync::Arc<Self>) -> std::sync::Arc<RWJson> {
        std::sync::Arc::clone(&self)
    }

    ///
    pub fn overwrite_all_episodes(
        self: &std::sync::Arc<Self>,
        series: &Series,
        episodes: Vec<Episode>,
    ) -> Result<()> {
        let rw_guard = self.rw_map.get(series).unwrap().lock();
        rw_guard.overwrite_all_episodes(episodes).unwrap();
        Ok(())
    }

    // ///
    // /// unused
    // pub(crate) fn push_episode(self: &std::sync::Arc<Self>, episode: Episode) -> Result<()> {
    //     let file_path_mutex = self.file_path_map.get(&episode.series()).unwrap();
    //     file_path_mutex.lock().push_episode(episode).unwrap();
    //     Ok(())
    // }

    ///
    pub(crate) fn read_all_episodes(
        self: &std::sync::Arc<Self>,
        series: &Series,
    ) -> Result<Vec<Episode>> {
        let rw_guard = self.rw_map.get(series).unwrap().lock();
        let episodes = rw_guard.read_all_episodes().unwrap();
        Ok(episodes)
    }

    // ///
    // /// unused
    // pub(crate) fn read_episodes_no_sha1(
    //     self: &std::sync::Arc<Self>,
    //     series: &Series,
    // ) -> Result<Vec<Episode>> {
    //     let all_episodes = self.read_all_episodes(series)?;
    //     let no_sha1_episodes = all_episodes
    //         .into_iter()
    //         .filter(|episode| episode.sha1().is_none())
    //         .collect::<Vec<Episode>>();
    //     Ok(no_sha1_episodes)
    // }

    ///
    pub(crate) fn edit_episode(self: &std::sync::Arc<Self>, episode: &Episode) -> Result<()> {
        let rw_guard = self.rw_map.get(&episode.series()).unwrap().lock();
        rw_guard.edit_episode(episode.clone())?;
        Ok(())
    }
}

///
struct RW {
    file_path: std::path::PathBuf,
}

///
impl RW {
    ///
    fn new(series: Series, dir_path: &std::path::Path) -> RW {
        let file_path = dir_path.join(series.data_json_filename());
        RW { file_path }
    }

    ///
    fn path(&self) -> &std::path::Path {
        self.file_path.as_path()
    }

    ///
    fn overwrite_all_episodes(&self, episodes: Vec<Episode>) -> Result<()> {
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

    // ///
    // /// unused
    // fn push_episode(&self, episode: Episode) -> Result<()> {
    //     let mut episodes = self.read_all_episodes().unwrap();
    //     episodes.push(episode);
    //     self.overwrite_all_episodes(episodes).unwrap();
    //     Ok(())
    // }

    ///
    fn read_all_episodes(&self) -> Result<Vec<Episode>> {
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

    fn edit_episode(&self, episode: Episode) -> Result<()> {
        let all = self.read_all_episodes().unwrap().into_iter();
        let mut filtered = all
            .filter(|ep| ep.id() != episode.id())
            .collect::<Vec<Episode>>();
        filtered.push(episode.clone());
        self.overwrite_all_episodes(filtered).unwrap();
        Ok(())
    }
}
