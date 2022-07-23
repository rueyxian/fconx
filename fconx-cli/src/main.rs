///
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

///
#[tokio::main]
async fn main() -> Result<()> {
    let (fconx, log_recv, canceller) = fconx::Fconx::new();

    cancel_handler(canceller);
    log_handler(log_recv);
    run(fconx).await?;

    println!("bye~");
    Ok(())
}

///
async fn run(fconx: fconx::Fconx) -> Result<()> {
    fconx.scrape_episodes().await?;
    fconx.scrape_download_url().await?;
    fconx.download_mp3().await?;
    Ok(())
}

///
fn cancel_handler(cancel: std::sync::Arc<fconx::Canceller>) {
    let (tx, rx) = flume::bounded(0);

    ctrlc::set_handler(move || {
        println!("\nshut down... ");
        tx.send(()).unwrap();
    })
    .unwrap();

    tokio::spawn(async move {
        rx.recv().unwrap();
        cancel.cancel();
    });
}

///
fn log_handler(log_recv: flume::Receiver<fconx::Log>) {
    use fconx::Log;

    tokio::spawn(async move {
        while let Ok(recv) = log_recv.recv() {
            match recv {
                //
                Log::NewEpisodes { series, episodes } => {
                    println!("found {} new episodes in {:?}", episodes.len(), series);
                }

                // ========================

                //
                Log::ToScrape { episodes } => {
                    println!("{} episodes to scrape", episodes.len());
                }

                //
                Log::ScrapeDownloadUrlStart { idx, episode } => {
                    println!(
                        "{:02} start scrape: {:?} {} {}",
                        idx,
                        episode.series(),
                        episode.number(),
                        episode.title()
                    );
                }

                //
                Log::ScrapeDownloadUrlDone { idx: _, episode: _ } => (),

                //
                Log::ScrapeDownloadUrlError { idx, episode } => {
                    println!(
                        "{:02} SCRAPE ERROR: {:?} {} {}",
                        idx,
                        episode.series(),
                        episode.number(),
                        episode.title()
                    );
                }

                //
                Log::ScrapeDownloadUrlThreadKill { idx } => {
                    println!("{:02} thread kill", idx,);
                }

                // ========================

                //
                Log::ExistingMp3Found {
                    idx,
                    series,
                    file_path,
                } => {
                    println!(
                        "{:02} found: {:?} {}",
                        idx,
                        series,
                        file_path.file_name().unwrap().to_string_lossy()
                    );
                }

                // ========================

                //
                Log::ToDownload { episodes } => {
                    println!("{} episodes to download", episodes.len());
                }

                //
                Log::DownloadStart { idx, episode } => {
                    println!(
                        "{:02} start download: {:?} {} {}",
                        idx,
                        episode.series(),
                        episode.number(),
                        episode.title()
                    );
                }

                //
                Log::DownloadDone { idx: _, episode: _ } => {}

                //
                Log::DownloadError { idx, episode } => {
                    println!(
                        "{:02} DOWNLOAD ERROR: {:?} {} {}",
                        idx,
                        episode.series(),
                        episode.number(),
                        episode.title()
                    );
                }

                //
                Log::DownloadThreadKill { idx } => {
                    println!("{:02} thread kill", idx,);
                } // ========================
            }
        }
    });
}
