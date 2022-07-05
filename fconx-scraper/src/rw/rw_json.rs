// ========================
// ===============================================
// ===============================================================================================

use crate::config::Config;
use crate::config::Series;
use crate::episode::Episode;

// ===============================================================================================

pub struct RWJson {
    file_path_map: std::collections::HashMap<Series, parking_lot::Mutex<FilePath>>,
}

impl RWJson {
    // pub fn new_arc(dir_path: &std::path::Path, series_vec: &Vec<Series>) -> std::sync::Arc<RWJson> {
    //     // std::fs::create_dir_all(".data/").unwrap();
    //     let dir_path = dir_path.join(".data");
    //     let mut file_path_map = std::collections::HashMap::with_capacity(series_vec.len());
    //     for &series in series_vec {
    //         let file_path = FilePath::new(series, dir_path.as_path());
    //         // let file_path = FilePath::new(series, dir_path);
    //         let file_path = parking_lot::Mutex::new(file_path);
    //         file_path_map.insert(series, file_path);
    //     }
    //     std::sync::Arc::new(RWJson { file_path_map })
    // }

    pub fn new_arc(config: std::rc::Rc<Config>) -> std::sync::Arc<RWJson> {
        // std::fs::create_dir_all(".data/").unwrap();
        // let dir_path = dir_path.join(".data");
        let mut file_path_map = std::collections::HashMap::with_capacity(config.series_vec().len());
        for &series in config.series_vec().iter() {
            let file_path = FilePath::new(series, config.data_dir_path().as_path());
            // let file_path = FilePath::new(series, dir_path);
            let file_path = parking_lot::Mutex::new(file_path);
            file_path_map.insert(series, file_path);
        }
        std::sync::Arc::new(RWJson { file_path_map })
    }

    pub fn arc_clone(self: &std::sync::Arc<Self>) -> std::sync::Arc<RWJson> {
        std::sync::Arc::clone(&self)
    }

    pub fn write_episode(self: &std::sync::Arc<Self>, episode: Episode) -> anyhow::Result<()> {
        let path_mutex = self.file_path_map.get(&episode.series()).unwrap();
        path_mutex.lock().write_episode(episode).unwrap();
        Ok(())
    }

    pub fn read_episodes(
        self: &std::sync::Arc<Self>,
        series: &Series,
    ) -> anyhow::Result<Vec<Episode>> {
        let path_mutex = self.file_path_map.get(series).unwrap();
        let episodes = path_mutex.lock().read_episodes().unwrap();
        Ok(episodes)
    }
}

// ===============================================================================================

struct FilePath {
    file_path: std::path::PathBuf,
}

impl FilePath {
    fn new(series: Series, dir_path: &std::path::Path) -> FilePath {
        let file_path = dir_path.join(series.data_json_filename());
        FilePath { file_path }
    }

    fn path(&self) -> &std::path::Path {
        self.file_path.as_path()
    }

    fn write_episode(&self, episode: Episode) -> anyhow::Result<()> {
        use std::io::Write;

        let mut episodes = self.read_episodes().unwrap();

        episodes.push(episode);

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

    fn read_episodes(&self) -> anyhow::Result<Vec<Episode>> {
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

// ===============================================================================================
