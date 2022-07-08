use crate::config::Config;
use crate::config::Series;
use crate::episode::Episode;

///
#[derive(Debug)]
pub struct RWMp3 {
    workers: usize,
    dir_path_map: std::collections::HashMap<Series, std::path::PathBuf>,
}

///
impl RWMp3 {
    ///
    pub fn new_arc(config: std::rc::Rc<Config>, workers: usize) -> std::sync::Arc<RWMp3> {
        let mut dir_path_map = std::collections::HashMap::with_capacity(config.series_vec().len());
        for &series in config.series_vec().iter() {
            let dir_name = series.mp3_dirname();
            let dir_path = config.dir_path().join(dir_name);
            dir_path_map.insert(series, dir_path);
        }
        let rw_mp3 = RWMp3 {
            workers,
            dir_path_map,
        };
        std::sync::Arc::new(rw_mp3)
    }

    ///
    pub fn arc_clone(self: &std::sync::Arc<Self>) -> std::sync::Arc<RWMp3> {
        std::sync::Arc::clone(&self)
    }

    ///
    fn get_filename(episode: &Episode) -> String {
        format!("{} - {}.mp3", episode.number(), episode.title())
    }

    ///
    pub async fn write_mp3(
        self: &std::sync::Arc<Self>,
        episode: &Episode,
        bytes: bytes::Bytes,
    ) -> anyhow::Result<()> {
        let mut reader = std::io::Cursor::new(bytes);
        let mut writer = {
            let dir_path = self.dir_path_map.get(&episode.series()).unwrap();
            let file_name = RWMp3::get_filename(episode);
            let file_path = dir_path.join(file_name);
            tokio::fs::File::create(file_path).await.unwrap()
        };
        tokio::io::copy(&mut reader, &mut writer).await?;
        Ok(())
    }

    ///
    fn read_dir(self: &std::sync::Arc<Self>, series: Series) -> Vec<std::path::PathBuf> {
        let dir_path = std::path::Path::new(self.dir_path_map.get(&series).unwrap());
        let dir_entries = std::fs::read_dir(dir_path).unwrap();
        let mut out = Vec::new();
        for dir_entry in dir_entries {
            let dir_entry = dir_entry.unwrap();
            let metadata = dir_entry.metadata().unwrap();
            let filename = {
                let os_string = dir_entry.file_name().to_owned();
                format!("{}", os_string.to_string_lossy())
            };
            if !(metadata.is_file() && filename.ends_with(".mp3")) {
                continue;
            }
            let file_path = dir_entry.path();
            out.push(file_path);
        }
        out
    }

    ///
    // TODO: this process rather slow
    pub async fn read_mp3s_and_to_sha1(
        self: &std::sync::Arc<Self>,
        series: Series,
    ) -> anyhow::Result<Vec<String>> {
        use std::io::Read;

        let (file_paths_mutex, file_count) = {
            let v = self.read_dir(series);
            let count = v.len();
            (std::sync::Arc::new(parking_lot::Mutex::new(v)), count)
        };

        let out_mutex = {
            let v = Some(Vec::<String>::with_capacity(file_count));
            std::sync::Arc::new(parking_lot::Mutex::new(v))
        };

        let mut handles = Vec::with_capacity(self.workers);

        for _ in 0..usize::min(self.workers, file_count) {
            let file_paths_mutex = std::sync::Arc::clone(&file_paths_mutex);
            let out_mutex = std::sync::Arc::clone(&out_mutex);
            let h = tokio::spawn(async move {
                loop {
                    let file_path = {
                        file_paths_mutex.lock().pop() // drop the guard immediately
                    };
                    if let Some(file_path) = file_path {
                        println!("found {:?} {:?}", series, file_path.file_name().unwrap());
                        let mut file = std::fs::OpenOptions::new()
                            .read(true)
                            .write(true)
                            .create(true)
                            .open(file_path.as_path())
                            .unwrap();
                        let mut buf = Vec::new();
                        file.read_to_end(&mut buf).unwrap();
                        let sha1 = {
                            use crypto::digest::Digest;
                            let mut hasher = crypto::sha1::Sha1::new();
                            hasher.input(&buf[..]);
                            hasher.result_str()
                        };
                        {
                            let mut out_guard = out_mutex.lock();
                            let out = out_guard.as_mut().unwrap();
                            out.push(sha1);
                        }
                    } else {
                        break;
                    }
                }
            });
            handles.push(h);
        }

        for h in handles {
            h.await.unwrap();
        }

        let out = {
            let mut out_guard = out_mutex.lock();
            out_guard.take().unwrap()
        };

        Ok(out)
    }
}
