use crate::config::ConfigModel;
use crate::config::Series;

///
#[derive(Debug)]
pub(crate) struct Config {
    dir_path: std::sync::Arc<std::path::PathBuf>,
    data_dir_path: std::sync::Arc<std::path::PathBuf>,
    temp_dir_path: std::sync::Arc<std::path::PathBuf>,
    series_vec: std::sync::Arc<Vec<Series>>,
}

///
impl Config {
    ///
    const DATA_DIRNAME: &'static str = ".data";
    const TEMP_DIRNAME: &'static str = ".temp";

    ///
    fn read_config_file() -> ConfigModel {
        let config_dir_path = {
            let homedir = std::env::var("HOME").unwrap();
            std::path::Path::new(&homedir).join(".config/fconx/")
        };
        let config_file_path = config_dir_path.join("config.toml");

        match std::fs::read_to_string(config_file_path.as_path()) {
            Ok(file_str) => {
                // TODO: error handling
                toml::from_str::<ConfigModel>(&file_str).unwrap()
            }
            Err(_) => {
                std::fs::create_dir_all(config_dir_path).unwrap();
                let model = ConfigModel::default();
                let mut reader = {
                    let toml = toml::to_string_pretty(&model).unwrap();
                    std::io::Cursor::new(toml)
                };
                let mut writer = std::fs::File::create(config_file_path.as_path()).unwrap();
                std::io::copy(&mut reader, &mut writer).unwrap();
                model
            }
        }
    }

    ///
    pub(crate) fn new_arc() -> std::sync::Arc<Config> {
        let config_model = Config::read_config_file();

        let data_dir_path = {
            let path = config_model.dir_path.as_path().join(".data");
            std::sync::Arc::new(path)
        };

        let temp_dir_path = {
            let path = config_model.dir_path.as_path().join(".temp");
            std::sync::Arc::new(path)
        };

        // let dir_path = std::sync::Arc::new(dir_path);
        let dir_path = std::sync::Arc::new(config_model.dir_path);
        let series_vec = std::sync::Arc::new(config_model.series_vec);
        let config = Config {
            dir_path,
            data_dir_path,
            temp_dir_path,
            series_vec,
        };
        std::sync::Arc::new(config)
    }

    ///
    pub(crate) fn create_dirs(self: std::sync::Arc<Config>) -> std::sync::Arc<Self> {
        std::fs::create_dir_all(self.dir_path.as_path()).unwrap();

        // ../.data/
        let data_dir_path = self.dir_path.as_path().join(Config::DATA_DIRNAME);
        std::fs::create_dir_all(&data_dir_path).unwrap();

        // ../.temp/
        let temp_dir_path = self.dir_path.as_path().join(Config::TEMP_DIRNAME);
        std::fs::create_dir_all(&temp_dir_path).unwrap();

        // ../[series name]/
        for series in self.series_vec.iter() {
            let mp3_dir_name = series.mp3_dirname();
            let mp3_dir_path = self.dir_path.as_path().join(mp3_dir_name);
            std::fs::create_dir_all(&mp3_dir_path).unwrap();
        }

        self
    }

    ///
    pub(crate) fn arc_clone(self: &std::sync::Arc<Self>) -> std::sync::Arc<Config> {
        std::sync::Arc::clone(self)
    }

    ///
    pub(crate) fn dir_path(self: &std::sync::Arc<Self>) -> std::sync::Arc<std::path::PathBuf> {
        std::sync::Arc::clone(&self.dir_path)
    }

    ///
    pub(crate) fn data_dir_path(self: &std::sync::Arc<Self>) -> std::sync::Arc<std::path::PathBuf> {
        std::sync::Arc::clone(&self.data_dir_path)
    }

    ///
    pub(crate) fn temp_dir_path(self: &std::sync::Arc<Self>) -> std::sync::Arc<std::path::PathBuf> {
        std::sync::Arc::clone(&self.temp_dir_path)
    }

    ///
    pub(crate) fn series_vec(self: &std::sync::Arc<Self>) -> std::sync::Arc<Vec<Series>> {
        std::sync::Arc::clone(&self.series_vec)
    }
}
