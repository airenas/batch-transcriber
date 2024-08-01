use std::path::{Path, PathBuf};
use std::{error::Error, time::Duration};

use crate::filer::file::Filer;
use crate::{QProcessor, ASR_FILE_LAT, ASR_FILE_RES, DIR_FAILED, DIR_PROCESSED, DIR_WORKING};
use pgmq::Message;
use tokio::{select, time::sleep};
use tokio_util::sync::CancellationToken;

use super::client::ASRClient;
use crate::data::api::ResultMessage;
use crate::postgres::queue::PQueue;

pub struct Worker {
    filer: Filer,
    result_queue: PQueue,
    ct: CancellationToken,
    asr_client: ASRClient,
}

impl Worker {
    pub async fn new(
        ct: CancellationToken,
        asr_client: ASRClient,
        result_queue: PQueue,
        filer: Filer,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        log::info!("Init Result Worker");
        Ok(Self {
            filer,
            result_queue,
            ct,
            asr_client,
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        log::info!("Run result worker");
        loop {
            let mut was: bool = false;
            let res = self
                .result_queue
                .process(|msg: Message<ResultMessage>| async move { self.process_msg(msg).await })
                .await;
            match res {
                Ok(v) => {
                    was = v;
                }
                Err(e) => {
                    log::error!("{}", e);
                }
            }
            if self.ct.is_cancelled() {
                log::info!("Result Worker cancelled");
                break;
            }
            if !was {
                select! {
                    _ = self.ct.cancelled() => {
                        log::info!("Result Worker cancelled");
                        break;
                    }
                    _ = sleep(Duration::from_secs(1)) => { }
                }
            }
        }
        log::info!("Stop result worker");
        Ok(())
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
        self.filer
            .save_txt(&make_name(&f_name, ".err"), "failed", err_str)?;
        self.filer.move_to(&f_name, DIR_WORKING, DIR_FAILED)?;
        if let Err(e) = self
            .filer
            .move_to(&make_name(&f_name, ".info"), DIR_WORKING, DIR_FAILED)
        {
            log::info!("No info file?: {}", e);
        }
        Ok(())
    }

    async fn process_success(&self, msg_asr: ResultMessage) -> anyhow::Result<()> {
        log::info!("Process success {:?}", msg_asr);
        let f_name = msg_asr.file.clone();
        let res = self.load_res(&msg_asr.external_id, ASR_FILE_RES).await?;
        let res_lat = self.load_res(&msg_asr.external_id, ASR_FILE_LAT).await?;
        self.filer
            .save_txt(&make_name(&f_name, ".txt"), "processed", &res)?;
        self.filer
            .save_txt(&make_name(&f_name, ".lat"), "processed", &res_lat)?;
        self.filer.move_to(&f_name, DIR_WORKING, DIR_PROCESSED)?;
        if let Err(e) = self
            .filer
            .move_to(&make_name(&f_name, ".info"), DIR_WORKING, DIR_PROCESSED)
        {
            log::info!("No info file?: {}", e);
        }
        Ok(())
    }

    async fn load_res(&self, external_id: &str, file: &str) -> anyhow::Result<String> {
        self.asr_client.result(external_id, file).await
    }
}

fn make_name(f_name: &str, ext: &str) -> String {
    let path = Path::new(f_name);
    let mut new_path = PathBuf::from(path);
    let mut clean_ext = ext;
    while clean_ext.starts_with('.') {
        clean_ext = &clean_ext[1..]
    }
    new_path.set_extension(clean_ext);
    new_path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_case::test_case;

    #[test_case("document.wav", ".txt", "document.txt"; "change extension")]
    #[test_case("archive.tar.gz", ".lat", "archive.tar.lat"; "Change last extension in multi-extension file")]
    fn test_make_name(original: &str, new_ext: &str, expected: &str) {
        let actual = make_name(original, new_ext);
        assert_eq!(expected, actual);
    }
}
