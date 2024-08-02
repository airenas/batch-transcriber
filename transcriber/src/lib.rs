use async_trait::async_trait;
use pgmq::Message;
use std::future::Future;

pub mod asr;
pub mod data;
pub mod filer;
pub mod model;
pub mod postgres;

pub const INPUT_QUEUE: &str = "asr_input";
pub const RESULT_QUEUE: &str = "asr_result";
pub const CLEAN_QUEUE: &str = "asr_clean";

pub const DIR_INCOMMING: &str = "incomming";
pub const DIR_WORKING: &str = "working";
pub const DIR_PROCESSED: &str = "processed";
pub const DIR_FAILED: &str = "failed";

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
