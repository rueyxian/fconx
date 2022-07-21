// use anyhow::Result;
use tokio::sync::broadcast;


///
type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

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

    println!("poopaye~");
    Ok(())
}

///
async fn run(shutdown_send: broadcast::Sender<()>) -> Result<()> {
    let fconx = fconx_scraper::Fconx::new();

    fconx.scrape_episodes().await?;
    fconx.scrape_download_url().await?;
    fconx.download_mp3().await?;

    shutdown_send.send(()).unwrap();

    Ok(())
}
