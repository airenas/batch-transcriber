use crate::{data::api::ASRMessage, QSender};
use async_trait::async_trait;
use serde::Serialize;
use std::{error::Error, future::Future, time::Duration};

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

    pub async fn process<F, Fut>(&self, func: F) -> Result<bool, Box<dyn Error + Send + Sync>>
    where
        F: Fn(Message<ASRMessage>) -> Fut,
        Fut: Future<Output = Result<bool, Box<dyn Error + Send + Sync>>>,
    {
        let message: Option<Message<ASRMessage>> = self
            .pgmq
            .read::<ASRMessage>(&self.queue_name, Some(30))
            .await?;
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
impl<T: 'static> QSender<T> for PQueue
where
    T: Serialize + Send + Sync,
{
    async fn send(&self, message: T) -> Result<(), Box<dyn Error + Send + Sync>> {
        log::info!("Sending msg");
        let id: i64 = self
            .pgmq
            .send(&self.queue_name, &message)
            .await
            .map_err(|err| format!("Can't send: {}", err))?;
        log::info!("sent: {}", id);
        Ok(())
    }
}
