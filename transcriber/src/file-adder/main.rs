use std::error::Error;
use transcriber::filer::file::Filer;
use transcriber::postgres::queue::PQueue;

use clap::Parser;
// use super:: lib::filer::Filer;

/// Add audio task to to transcription queue
#[derive(Parser, Debug)]
#[command(version = env!("CARGO_APP_VERSION"), name = "file-adder", about, long_about = None)]
struct Args {
    /// File name
    #[arg(short, long, env)]
    file: String,

    /// Base working dir
    #[arg(short, long, env)]
    base_dir: String,

    /// Postgres SQL (QUEUE) connection string
    #[arg(short, long, env)]
    postgres_url: String,

    /// Only send msg to queue
    #[clap(long, short, action, env, default_value = "false")]
    only_msg: bool,
}

async fn main_int(args: Args) -> Result<(), Box<dyn Error>> {
    log::info!("Starting file adder");
    log::info!("Version      : {}", env!("CARGO_APP_VERSION"));
    log::info!("File         : {}", args.file);
    log::info!("Base dir     : {}", args.base_dir);

    log::info!("Connecting to postgres...");
    let pq = PQueue::new(args.postgres_url).await?;
    if !args.only_msg {
        let f = Filer::new(args.base_dir);
        f.move_working(args.file.as_str())?;
    } else {
        log::warn!("Skip copying file");
    }
    pq.add_job(args.file.as_str()).await?;

    log::info!("Done");
    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 1)]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    let args = Args::parse();
    if let Err(e) = main_int(args).await {
        log::error!("{}", e);
        return Err(e);
    }
    Ok(())
}
