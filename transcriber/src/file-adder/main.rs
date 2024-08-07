use std::path::PathBuf;
use transcriber::filer::file::{make_name, Filer};
use transcriber::postgres::queue::PQueue;
use transcriber::{data::api::ASRMessage, DIR_WORKING};

use clap::Parser;
use transcriber::{QSender, DIR_INCOMING, INFO_EXTENSION, INPUT_QUEUE};
use ulid::Ulid;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Add audio task to to transcription queue
#[derive(Parser, Debug)]
#[command(version = env!("CARGO_APP_VERSION"), name = "file-adder", about, long_about = None)]
struct Args {
    /// File name
    #[arg(short, long, env)]
    file: Option<String>,

    /// Base working dir
    #[arg(short, long, env)]
    base_dir: String,

    /// Server base working dir
    #[arg(short, long, env, default_value = "")]
    server_base_dir: String,

    /// Postgres SQL (QUEUE) connection string
    #[arg(short, long, env)]
    postgres_url: String,

    /// Only send msg to queue
    #[clap(long, short, action, env, default_value = "false")]
    only_msg: bool,

    /// Send all files from incoming
    #[arg(long, env, default_value = "false")]
    auto: bool,
}

async fn main_int(args: Args) -> anyhow::Result<()> {
    log::info!("Starting file adder");
    log::info!("Version      : {}", env!("CARGO_APP_VERSION"));
    log::info!("Base dir     : {}", args.base_dir);
    let file = args.file.clone().unwrap_or("".to_string());
    if args.auto {
        log::info!(
            "Add all      : {}",
            args.base_dir.clone() + "/" + DIR_INCOMING
        );
    }
    log::info!("Connecting to postgres...");
    let pq = PQueue::new(&args.postgres_url, INPUT_QUEUE)
        .await
        .map_err(anyhow::Error::msg)?;
    let sender = Box::new(pq) as Box<dyn QSender<ASRMessage>>;
    let f = Filer::new(&args.base_dir);
    let added = if args.auto {
        add_files(sender.as_ref(), &f, &args.base_dir, &args.server_base_dir, args.only_msg).await?
    } else {
        add_file(sender.as_ref(), &f, &file, &args.base_dir, &args.server_base_dir, args.only_msg).await?
    };
    if added == 0 {
        log::warn!("No files to transcribe");
    } else {
        log::info!("Sent {} files to transcribe", added);
    }
    Ok(())
}

async fn add_file(
    sender: &dyn QSender<ASRMessage>,
    f: &Filer,
    file: &str,
    base_dir: &str,
    server_base_dir: &str,
    only_msg: bool,
) -> anyhow::Result<i64> {
    log::info!("Add file     : {}", file);
    let mut new_f_name = file.to_string();
    if !only_msg {
        new_f_name = f.non_existing_name(file, DIR_WORKING)?;
        f.move_to(file, &new_f_name, DIR_INCOMING, DIR_WORKING)?;
        if let Err(e) = f.move_to(
            &make_name(file, INFO_EXTENSION),
            &make_name(&new_f_name, INFO_EXTENSION),
            DIR_INCOMING,
            DIR_WORKING,
        ) {
            log::info!("No info file?: {}", e);
        }
    } else {
        log::warn!("Skip copying file");
    }
    let ulid = Ulid::new();
    let mut s_dir = server_base_dir;
    if s_dir.is_empty() {
        s_dir = base_dir;
    }
    sender
        .send(ASRMessage {
            file: new_f_name,
            id: ulid.to_string(),
            base_dir: s_dir.to_string(),
        })
        .await?;
    Ok(1)
}

async fn add_files(
    sender: &dyn QSender<ASRMessage>,
    f: &Filer,
    base_dir: &str,
    server_base_dir: &str,
    only_msg: bool,
) -> anyhow::Result<i64> {
    let mut source_path = PathBuf::from(base_dir);
    source_path.extend(&[DIR_INCOMING]);
    log::info!("checking dir     : {}", source_path.display());
    let mut res = 0;
    for entry in std::fs::read_dir(source_path)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(ext) = path.extension() {
                let ext_str = ext.to_str().unwrap_or("").to_lowercase();
                if ext_str == "mp3" || ext_str == "wav" || ext_str == "m4a" {
                    let file = path.file_name().unwrap().to_str().unwrap();
                    res += add_file(sender, f, file, base_dir, server_base_dir, only_msg).await?;
                }
            }
        }
    }
    Ok(res)
}

#[tokio::main(flavor = "multi_thread", worker_threads = 1)]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::Layer::default().compact())
        .init();
    let args = Args::parse();
    if let Err(e) = main_int(args).await {
        log::error!("{}", e);
        return Err(e);
    }
    Ok(())
}
