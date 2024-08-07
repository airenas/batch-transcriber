use async_trait::async_trait;
use pgmq::Message;
use std::future::Future;
use tokio::signal;

pub mod asr;
pub mod data;
pub mod filer;
pub mod model;
pub mod postgres;

pub const INPUT_QUEUE: &str = "asr_input";
pub const RESULT_QUEUE: &str = "asr_result";
pub const CLEAN_QUEUE: &str = "asr_clean";

pub const DIR_INCOMING: &str = "incoming";
pub const DIR_WORKING: &str = "working";
pub const DIR_PROCESSED: &str = "processed";
pub const DIR_FAILED: &str = "failed";
pub const INFO_EXTENSION: &str = ".meta";

pub const ASR_FILE_RES: &str = "resultFinal.txt";
pub const ASR_FILE_LAT: &str = "lat.restored.txt";

#[async_trait]
pub trait QSender<T>
where
    T: Send + Sync,
{
    async fn send(&self, data: T) -> anyhow::Result<()>;
}

#[async_trait]
pub trait QProcessor<T>
where
    T: Send + Sync,
{
    async fn process<F, Fut>(&self, func: F) -> anyhow::Result<bool>
    where
        F: Fn(Message<T>) -> Fut + Send,
        Fut: Future<Output = anyhow::Result<bool>> + Send;
}

pub async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {
            log::info!("Ctrl-C received, shutting down");
        },
        _ = terminate => {
            log::info!("SIGTERM received, shutting down");
        },
    }
}
