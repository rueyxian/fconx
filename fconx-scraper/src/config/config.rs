use crate::config::Series;

///
pub struct Config {
    // config_file_path: std::path::Buf,
    dir_path: std::rc::Rc<std::path::PathBuf>,
    data_dir_path: std::rc::Rc<std::path::PathBuf>,
    series_vec: std::rc::Rc<Vec<Series>>,
}

///
impl Config {
    ///
    const DATA_DIRNAME: &'static str = ".data";

    ///
    pub fn new_rc(dir_path: std::path::PathBuf, series_vec: Vec<Series>) -> std::rc::Rc<Config> {
        let data_dir_path = dir_path.as_path().join(Config::DATA_DIRNAME);
        let dir_path = std::rc::Rc::new(dir_path);
        let data_dir_path = std::rc::Rc::new(data_dir_path);
        let series_vec = std::rc::Rc::new(series_vec);
        let config = Config {
            dir_path,
            data_dir_path,
            series_vec,
        };
        std::rc::Rc::new(config)
    }

    ///
    pub fn create_dirs(self: &std::rc::Rc<Config>) -> std::rc::Rc<Self> {
        std::fs::create_dir_all(self.dir_path.as_path()).unwrap();

        let data_dir_path = self.dir_path.as_path().join(Config::DATA_DIRNAME);
        std::fs::create_dir_all(&data_dir_path).unwrap();

        for series in self.series_vec.iter() {
            let mp3_dir_name = series.mp3_dirname();
            let mp3_dir_path = self.dir_path.as_path().join(mp3_dir_name);
            std::fs::create_dir_all(&mp3_dir_path).unwrap();
        }

        self.to_owned()
    }

    ///
    pub fn rc_clone(self: &std::rc::Rc<Self>) -> std::rc::Rc<Config> {
        std::rc::Rc::clone(self)
    }

    ///
    pub fn dir_path(self: &std::rc::Rc<Self>) -> std::rc::Rc<std::path::PathBuf> {
        std::rc::Rc::clone(&self.dir_path)
    }

    ///
    pub fn data_dir_path(self: &std::rc::Rc<Self>) -> std::rc::Rc<std::path::PathBuf> {
        std::rc::Rc::clone(&self.data_dir_path)
    }

    ///
    pub fn series_vec(self: &std::rc::Rc<Self>) -> std::rc::Rc<Vec<Series>> {
        std::rc::Rc::clone(&self.series_vec)
    }
}

