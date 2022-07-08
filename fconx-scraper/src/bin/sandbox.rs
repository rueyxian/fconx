use std::time::Instant;

use fconx_scraper::config::Config;
use fconx_scraper::config::Series;
use fconx_scraper::rw::RWMp3;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    //

    let config = config().create_dirs();
    let rw_mp3 = RWMp3::new_arc(config, 256);

    let start = Instant::now();
    let sha1s = rw_mp3.read_mp3s_and_to_sha1(Series::OL).await?;
    // rw_mp3.read_mp3s(Series::OL).await?;
    for sha1 in sha1s {
        println!("{:?}", sha1);
    }
    println!("{:?}", start.elapsed());

    Ok(())
}

// ===============================================================================================

fn config() -> std::rc::Rc<Config> {
    let dir_path = {
        let homedir = std::env::var("HOME").unwrap();
        let dir_path = std::path::Path::new(&homedir);
        dir_path.join("Music/fconx/")
    };

    let series_vec = vec![
        // Series::FR,
        // Series::NSQ,
        // Series::PIMA,
        // Series::FMD,
        Series::OL,
    ];

    Config::new_rc(dir_path, series_vec)
}

// ===============================================================================================


