use deadpool_diesel::postgres::{Manager, Pool};
use deadpool_diesel::Runtime;
use std::error::Error;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;
use transcriber::asr::client::ASRClient;
use transcriber::asr::{clean_worker, res_worker, worker};
use transcriber::filer::file::Filer;
use transcriber::postgres::queue::PQueue;
use transcriber::{CLEAN_QUEUE, INPUT_QUEUE, RESULT_QUEUE};

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

    /// Background worker count
    #[arg(short, long, env, default_value = "1")]
    worker_count: i32,

    /// ASR URL
    #[arg(long, env)]
    asr_url: String,

    /// ASR auth key
    #[arg(long, env, default_value = "")]
    asr_auth_key: String,

    /// ASR recognizer
    #[arg(long, env, default_value = "ben")]
    asr_recognizer: String,
}

async fn main_int(args: Args) -> Result<(), Box<dyn Error + Send + Sync>> {
    log::info!("Starting file adder");
    log::info!("Version      : {}", env!("CARGO_APP_VERSION"));
    log::info!("Base dir     : {}", args.base_dir);
    log::info!("ASR URL      : {}", args.asr_url);
    log::info!("ASR Model    : {}", args.asr_recognizer);

    let f = Filer::new(args.base_dir);
    log::info!("Connecting to postgres...");
    let pq = PQueue::new(&args.postgres_url, INPUT_QUEUE).await?;
    let pq_res = PQueue::new(&args.postgres_url, RESULT_QUEUE).await?;
    let pq_clean = PQueue::new(&args.postgres_url, CLEAN_QUEUE).await?;

    let manager = Manager::new(args.postgres_url, Runtime::Tokio1);
    let pool = Pool::builder(manager).max_size(8).build()?;
    let asr_client = ASRClient::new(&args.asr_url, &args.asr_auth_key, &args.asr_recognizer)?;
    let token = CancellationToken::new();

    let tracker = TaskTracker::new();

    for i in 0..args.worker_count {
        let worker = worker::Worker::new(
            i,
            token.clone(),
            pool.clone(),
            asr_client.clone(),
            Box::new(pq_res.clone()),
            pq.clone(),
        )
        .await?;
        tracker.spawn(async move {
            if let Err(e) = worker.run().await {
                log::error!("{}", e);
            }
        });
    }
    let worker = res_worker::Worker::new(
        token.clone(),
        asr_client.clone(),
        pq_res,
        f,
        Box::new(pq_clean.clone()),
    )
    .await?;
    tracker.spawn(async move {
        if let Err(e) = worker.run().await {
            log::error!("{}", e);
        }
    });
    let worker = clean_worker::Worker::new(token.clone(), asr_client.clone(), pq_clean).await?;
    tracker.spawn(async move {
        if let Err(e) = worker.run().await {
            log::error!("{}", e);
        }
    });

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
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env_logger::init();
    let args = Args::parse();
    if let Err(e) = main_int(args).await {
        log::error!("{}", e);
        return Err(e);
    }
    Ok(())
}
