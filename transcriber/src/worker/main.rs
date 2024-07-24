use deadpool_diesel::postgres::{Manager, Pool};
use deadpool_diesel::Runtime;
use std::error::Error;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use transcriber::asr::worker;
use transcriber::filer::file::Filer;
use transcriber::postgres::queue::PQueue;

use clap::Parser;
// use super:: lib::filer::Filer;

/// ASR Worker
#[derive(Parser, Debug)]
#[command(version = env!("CARGO_APP_VERSION"), name = "asr-worker", about, long_about = None)]
struct Args {
    /// Base working dir
    #[arg(short, long, env)]
    base_dir: String,

    /// Postgres SQL (QUEUE) connection string
    #[arg(short, long, env)]
    postgres_url: String,
}

async fn main_int(args: Args) -> Result<(), Box<dyn Error>> {
    log::info!("Starting file adder");
    log::info!("Version      : {}", env!("CARGO_APP_VERSION"));
    log::info!("Base dir     : {}", args.base_dir);

    let f = Filer::new(args.base_dir);
    log::info!("Connecting to postgres...");
    let pq = PQueue::new(args.postgres_url.clone()).await?;

    let manager = Manager::new(args.postgres_url, Runtime::Tokio1);
    let pool = Pool::builder(manager).max_size(8).build()?;
    let token = CancellationToken::new();

    let tracker = TaskTracker::new();

    for i in 0..1 {
        let worker =
            worker::Worker::new(pq.clone(), f.clone(), i, token.clone(), pool.clone()).await?;
        tracker.spawn(async move {
            if let Err(e) = worker.run().await {
                log::error!("{}", e);
            }
        });
    }

    tracker.close();

    match signal::ctrl_c().await {
        Ok(()) => {
            log::info!("Cancel...");
            token.cancel()
        }
        Err(err) => {
            return Err(format!("Error registering signal handler: {}", err).into());
        }
    }

    // Wait for everything to finish.
    tracker.wait().await;

    log::info!("Done");
    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 4)]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args = Args::parse();
    if let Err(e) = main_int(args).await {
        log::error!("{}", e);
        return Err(e);
    }
    Ok(())
}
