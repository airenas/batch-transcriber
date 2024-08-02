use std::error::Error;

use crate::filer::file::{make_name, Filer};
use crate::{
    QSender, ASR_FILE_LAT, ASR_FILE_RES, DIR_FAILED, DIR_PROCESSED, DIR_WORKING, INFO_EXTENSION,
};
use pgmq::Message;
use tokio_util::sync::CancellationToken;

use super::client::ASRClient;
use crate::data::api::{CleanMessage, ResultMessage};
use crate::postgres::queue::PQueue;

pub struct Worker {
    filer: Filer,
    result_queue: PQueue,
    ct: CancellationToken,
    asr_client: ASRClient,
    clean_queue: Box<dyn QSender<CleanMessage> + Send + Sync>,
}

impl Worker {
    pub async fn new(
        ct: CancellationToken,
        asr_client: ASRClient,
        result_queue: PQueue,
        filer: Filer,
        clean_queue: Box<dyn QSender<CleanMessage> + Send + Sync>,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        log::info!("Init Result Worker");
        Ok(Self {
            filer,
            result_queue,
            ct,
            asr_client,
            clean_queue,
        })
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        crate::postgres::queue::run(
            self.result_queue.clone(),
            |msg: Message<ResultMessage>| async move { self.process_msg(msg).await },
            self.ct.clone(),
            "result worker",
        )
        .await
    }

    pub async fn process_msg(&self, msg: Message<ResultMessage>) -> anyhow::Result<bool> {
        log::info!("Process {:?}", msg);
        let msg_asr = msg.message;
        if msg.read_ct > 3 {
            log::warn!("Max retries reached {:?}", msg_asr);

            // don't fail here, just try set error
            if let Err(err) = self
                .process_error(
                    &msg_asr,
                    format!("Max retries reached {:?}", msg_asr).as_str(),
                )
                .await
            {
                log::error!("Error processing result message: {}", err);
            }

            return Ok(true);
        }
        if !msg_asr.finished {
            log::warn!("Skip non finished event {:?}", msg_asr);
            return Ok(true);
        }
        if let Some(err_str) = &msg_asr.error {
            self.process_error(&msg_asr, err_str).await?;
            return Ok(true);
        }
        self.process_success(msg_asr).await?;
        log::info!("done: {}", msg.msg_id);
        Ok(true)
    }

    async fn process_error(&self, msg_asr: &ResultMessage, err_str: &str) -> anyhow::Result<()> {
        log::info!("Process error {:?}", msg_asr);
        let f_name = msg_asr.file.clone();
        let new_f_name = self.filer.non_existing_name(&f_name, DIR_FAILED)?;
        self.filer
            .save_txt(&make_name(&new_f_name, ".err"), DIR_FAILED, err_str)?;
        self.filer
            .move_to(&f_name, &new_f_name, DIR_WORKING, DIR_FAILED)?;
        if let Err(e) = self.filer.move_to(
            &make_name(&f_name, INFO_EXTENSION),
            &make_name(&new_f_name, INFO_EXTENSION),
            DIR_WORKING,
            DIR_FAILED,
        ) {
            log::info!("No info file?: {}", e);
        }
        self.send_clean_msg(&msg_asr.external_id).await
    }

    async fn process_success(&self, msg_asr: ResultMessage) -> anyhow::Result<()> {
        log::info!("Process success {:?}", msg_asr);
        let f_name = msg_asr.file.clone();
        let res = self.load_res(&msg_asr.external_id, ASR_FILE_RES).await?;
        let res_lat = self.load_res(&msg_asr.external_id, ASR_FILE_LAT).await?;
        let new_f_name = self.filer.non_existing_name(&f_name, DIR_PROCESSED)?;
        self.filer
            .save_txt(&make_name(&new_f_name, ".txt"), DIR_PROCESSED, &res)?;
        self.filer
            .save_txt(&make_name(&new_f_name, ".lat"), DIR_PROCESSED, &res_lat)?;
        self.filer
            .move_to(&f_name, &new_f_name, DIR_WORKING, DIR_PROCESSED)?;
        if let Err(e) = self.filer.move_to(
            &make_name(&f_name, INFO_EXTENSION),
            &make_name(&new_f_name, INFO_EXTENSION),
            DIR_WORKING,
            DIR_PROCESSED,
        ) {
            log::info!("No info file?: {}", e);
        }
        self.send_clean_msg(&msg_asr.external_id).await
    }

    async fn load_res(&self, external_id: &str, file: &str) -> anyhow::Result<String> {
        self.asr_client.result(external_id, file).await
    }

    async fn send_clean_msg(&self, external_id: &str) -> anyhow::Result<()> {
        self.clean_queue
            .send(CleanMessage {
                external_id: external_id.to_string(),
            })
            .await
    }
}
