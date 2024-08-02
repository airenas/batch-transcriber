use crate::{data::api::ASRMessage, QProcessor, QSender};
use anyhow::Context;
use async_trait::async_trait;
use serde::Serialize;
use std::{error::Error, future::Future, time::Duration};
use tokio::{select, time::sleep};
use tokio_util::sync::CancellationToken;

use pgmq::{Message, PGMQueue};

#[derive(Clone)]
pub struct PQueue {
    pgmq: PGMQueue,
    queue_name: String,
}

impl PQueue {
    pub async fn new(p_url: &str, queue_name: &str) -> Result<Self, Box<dyn Error + Send + Sync>> {
        log::info!("Init PGMQ, with name: {queue_name}");
        let queue: PGMQueue = PGMQueue::new(p_url.to_string())
            .await
            .map_err(|err| format!("Can't connect to postgres: {}", err))?;
        let my_queue = queue_name.to_string();
        queue
            .create(&my_queue)
            .await
            .map_err(|err| format!("Can't create queue '{my_queue}': {}", err))?;
        Ok(Self {
            pgmq: queue,
            queue_name: my_queue,
        })
    }

    pub async fn mark_working(&self, id: i64) -> Result<(), Box<dyn Error>> {
        log::info!("Updating msg: {:?}", id);
        let vt = chrono::Utc::now() + Duration::from_secs(60);
        let message: Option<Message<ASRMessage>> = self
            .pgmq
            .set_vt(&self.queue_name, id, vt)
            .await
            .map_err(|err| format!("Can't set: {}", err))?;
        log::info!("updated: {:?}", message);
        Ok(())
    }
}

#[async_trait]
impl<T: 'static + for<'de> serde::Deserialize<'de> + std::fmt::Debug> QProcessor<T> for PQueue
where
    T: Send + Sync,
{
    async fn process<F, Fut>(&self, func: F) -> anyhow::Result<bool>
    where
        F: Fn(Message<T>) -> Fut + Send,
        Fut: Future<Output = anyhow::Result<bool>> + Send,
    {
        let message: Option<Message<T>> = self.pgmq.read::<T>(&self.queue_name, Some(30)).await?;
        match message {
            Some(msg) => {
                log::info!("Got msg: {:?}", msg);
                let id = msg.msg_id;
                let res = func(msg).await;
                match res {
                    Ok(delete) => {
                        if delete {
                            self.pgmq.delete(&self.queue_name, id).await?;
                            log::info!("processed: {:?}", id);
                        }
                    }
                    Err(e) => {
                        log::error!("Error: {}", e);
                    }
                }
                Ok(true)
            }
            None => Ok(false),
        }
    }
}

#[async_trait]
impl<T: 'static + std::fmt::Debug> QSender<T> for PQueue
where
    T: Serialize + Send + Sync,
{
    async fn send(&self, message: T) -> anyhow::Result<()> {
        log::info!("Sending msg {:?}", message);
        let id: i64 = self
            .pgmq
            .send(&self.queue_name, &message)
            .await
            .with_context(|| "Can't send")?;
        log::info!("sent: {}", id);
        Ok(())
    }
}

pub async fn run<
    T: 'static + for<'de> serde::Deserialize<'de> + std::fmt::Debug + Send + Sync,
    F,
    Fut,
>(
    queue: PQueue,
    func: F,
    ct: CancellationToken,
    name: &str,
) -> anyhow::Result<()>
where
    F: Fn(Message<T>) -> Fut + Send + Sync,
    Fut: Future<Output = anyhow::Result<bool>> + Send,
{
    log::info!("Run: {}", name);
    loop {
        let mut was: bool = false;
        let res = queue
            .process(&func)
            .await;
        match res {
            Ok(v) => {
                was = v;
            }
            Err(e) => {
                log::error!("{}", e);
            }
        }
        if ct.is_cancelled() {
            log::info!("cancelled: {}", name);
            break;
        }
        if !was {
            select! {
                _ = ct.cancelled() => {
                    log::info!("cancelled: {}", name);
                    break;
                }
                _ = sleep(Duration::from_secs(1)) => { }
            }
        }
    }
    log::info!("Stop: {}", name);
    Ok(())
}
