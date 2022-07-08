use anyhow::Result;
use fconx_scraper::rw::RWMp3;
use tokio::sync::broadcast;

use fconx_scraper::config::Config;
use fconx_scraper::config::Series;
use fconx_scraper::downloader;
use fconx_scraper::rw::RWJson;
use fconx_scraper::scraper;

///
#[tokio::main]
async fn main() -> Result<()> {
    let (shutdown_send, mut shutdown_recv) = broadcast::channel::<()>(256);

    {
        let shutdown_send = shutdown_send.clone();
        tokio::spawn(async move {
            match tokio::signal::ctrl_c().await {
                Ok(()) => {
                    shutdown_send.send(()).unwrap();

                    println!("\nshutting down...");
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    std::process::exit(0);
                }
                Err(err) => {
                    eprintln!("Unable to listen for shutdown signal: {}", err);
                    std::process::exit(1);
                }
            }
        });
    }

    run(shutdown_send.clone()).await?;

    // if Err(e) = run(shutdown_send.clone()).await {
    //     return e;
    // };

    shutdown_recv.recv().await.unwrap();

    println!("bye");
    Ok(())
}

///
async fn run(shutdown_send: broadcast::Sender<()>) -> Result<()> {
    let config = config().create_dirs();
    let rw_json = RWJson::new_arc(config.rc_clone());
    let rw_mp3 = RWMp3::new_arc(config.rc_clone(), 256);

    let scraper = scraper::Scraper::new(16, config.series_vec(), rw_json.arc_clone());
    scraper.run().await?;
    println!("scraping is done");

    let downloader = downloader::Downloader::new(
        16,
        config.series_vec(),
        rw_json.arc_clone(),
        rw_mp3.arc_clone(),
    );
    downloader.run().await?;
    println!("download is done");

    shutdown_send.send(()).unwrap();

    Ok(())
}

///
fn config() -> std::rc::Rc<Config> {
    let dir_path = {
        let homedir = std::env::var("HOME").unwrap();
        let dir_path = std::path::Path::new(&homedir);
        dir_path.join("Music/fconx/")
    };

    let series_vec = vec![
        // Series::FR,
        Series::NSQ,
        // Series::PIMA,
        // Series::FMD,
        // Series::OL,
    ];

    Config::new_rc(dir_path, series_vec)
}

