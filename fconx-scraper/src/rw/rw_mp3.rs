// ========================
// ===============================================
// ===============================================================================================

// use crate::episode::Episode;
// use crate::rw::RWJson;
// use crate::series::Series;
//
// pub struct RWMp3 {
//     file_path_map: std::collections::HashMap<String, parking_lot::Mutex<FilePath>>,
// }
//
// impl RWMp3 {
//     //
//     pub fn new_arc(
//         rw_json: std::sync::Arc<RWJson>,
//         dir_path: &std::path::Path,
//         series_vec: &Vec<Series>,
//     ) -> std::sync::Arc<RWMp3> {
//         let mut file_path_map = std::collections::HashMap::with_capacity(series_vec.len());
//         for &series in series_vec {
//             let dir_path = dir_path.join(series.mp3_dirname());
//
//
//
//             let file_path = FilePath::new(series, dir_path.as_path());
//             let file_path = parking_lot::Mutex::new(file_path);
//             file_path_map.insert(series, file_path);
//         }
//         std::sync::Arc::new(RWMp3 { file_path_map })
//     }
//
//     pub fn write_mp3(self: &std::sync::Arc<Self>, episode: Episode) -> anyhow::Result<()> {
//         // let path_mutex = self.dir_path_map.get()
//         Ok(())
//     }
// }

// ===============================================================================================

// struct FilePath {
//     file_path: std::path::PathBuf,
// }
//
// impl FilePath {
//     //
//     fn new(series: Series, dir_path: &std::path::Path) -> FilePath {
//         let dir_path = dir_path.join(series.mp3_dirname());
//         FilePath {
//             file_path: dir_path,
//         }
//     }
//
//     // pub fn write_mp3(&self, )
// }
