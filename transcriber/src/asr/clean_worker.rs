use pgmq::Message;
use tokio_util::sync::CancellationToken;

use super::client::ASRClient;
use crate::data::api::CleanMessage;
use crate::postgres::queue::PQueue;

pub struct Worker {
    queue: PQueue,
    ct: CancellationToken,
    asr_client: ASRClient,
}

impl Worker {
    pub async fn new(
        ct: CancellationToken,
        asr_client: ASRClient,
        queue: PQueue,
    ) -> anyhow::Result<Self> {
        log::info!("Init Result Worker");
        Ok(Self {
            queue,
            ct,
            asr_client,
        })
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        crate::postgres::queue::run(
            self.queue.clone(),
            |msg: Message<CleanMessage>| async move { self.process_msg(msg).await },
            self.ct.clone(),
            "clean worker",
        )
        .await
    }

    pub async fn process_msg(&self, msg: Message<CleanMessage>) -> anyhow::Result<bool> {
        log::info!("Process {:?}", msg);
        let msg_asr = msg.message;
        if msg.read_ct > 3 {
            log::error!("Max retries reached {:?}", msg_asr);
            return Ok(true);
        }
        self.clean(&msg_asr.external_id).await?;
        log::info!("done: {}", msg.msg_id);
        Ok(true)
    }

    async fn clean(&self, id: &str) -> anyhow::Result<()> {
        log::info!("Call clean {}", id);
        self.asr_client.clean(id).await
    }
}
