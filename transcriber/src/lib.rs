use async_trait::async_trait;
use std::error::Error;

pub mod asr;
pub mod data;
pub mod filer;
pub mod model;
pub mod postgres;

pub const INPUT_QUEUE: &str = "asr_input";
pub const RESULT_QUEUE: &str = "asr_result";

#[async_trait]
pub trait QSender<T>
where
    T: Send + Sync,
{
    async fn send(&self, data: T) -> Result<(), Box<dyn Error + Send + Sync>>;
}
