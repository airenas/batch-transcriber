pub mod handler;
use std::time::Duration;
use tokio::net::TcpListener;

use clap::Parser;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tower_http::{limit::RequestBodyLimitLayer, timeout::TimeoutLayer};

use axum::{
    extract::DefaultBodyLimit,
    routing::{get, post},
    Router,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use transcriber::filer::file::Filer;
use transcriber::shutdown_signal;

/// Sound saver http service
#[derive(Parser, Debug)]
#[command(version = env!("CARGO_APP_VERSION"), name = "sound-keeper", about, long_about = None)]
struct Args {
    /// Base working dir
    #[arg(short, long, env)]
    base_dir: String,

    /// Server port
    #[arg(long, env, default_value = "8000")]
    port: i32,
}

async fn main_int(args: Args) -> anyhow::Result<()> {
    log::info!("Starting file adder");
    tracing::info!(version = env!("CARGO_APP_VERSION"));
    tracing::info!(dir = args.base_dir, "base dir");
    tracing::info!(port = args.port, "port");
    log::info!("Init tracing...");

    log::info!("Connecting to postgres...");
    let f = Filer::new(&args.base_dir);

    let cors = CorsLayer::new()
        .allow_methods([axum::http::Method::GET, axum::http::Method::POST])
        .allow_origin(Any);

    let app = Router::new()
        .route("/live", get(handler::live::handler))
        .route("/upload", post(handler::upload::handler))
        .layer(DefaultBodyLimit::disable())
        .layer(RequestBodyLimitLayer::new(500 * 1024 * 1024))
        .with_state(f)
        .layer((
            TraceLayer::new_for_http(),
            TimeoutLayer::new(Duration::from_secs(40)),
            cors,
        ));

    let listener = TcpListener::bind(format!("0.0.0.0:{}", args.port)).await?;

    tracing::info!(port = args.port, "starting http");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    tracing::info!("Bye");
    Ok(())
}

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
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
