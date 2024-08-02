use std::time::Instant;
use std::{error::Error, time::Duration};

use deadpool_diesel::postgres::Pool;
use diesel::query_dsl::methods::FilterDsl;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use pgmq::Message;
use rand::Rng;
use tokio::{task::JoinHandle, time::sleep};
use tokio_util::sync::CancellationToken;

use crate::data::api::ResultMessage;
use crate::postgres::queue::PQueue;
use crate::QSender;
use crate::{
    data::api::ASRMessage,
    model::{
        models::WorkData,
        schema::{self},
    },
};

use super::client::ASRClient;

pub struct Worker {
    result_queue: Box<dyn QSender<ResultMessage> + Send + Sync>,
    input_queue: PQueue,
    id: i32,
    ct: CancellationToken,
    pool: Pool,
    asr_client: ASRClient,
}

impl Worker {
    pub async fn new(
        id: i32,
        ct: CancellationToken,
        pool: Pool,
        asr_client: ASRClient,
        result_queue: Box<dyn QSender<ResultMessage> + Send + Sync>,
        input_queue: PQueue,
    ) -> Result<Self, Box<dyn Error + Send + Sync>> {
        log::info!("Init Worker");
        Ok(Self {
            input_queue,
            id,
            ct,
            pool,
            asr_client,
            result_queue,
        })
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        crate::postgres::queue::run(
            self.input_queue.clone(),
            |msg: Message<ASRMessage>| async move { self.process_msg(msg).await },
            self.ct.clone(),
            format!("worker {}", self.id).as_str(),
        )
        .await
    }

    pub async fn process_msg(&self, msg: Message<ASRMessage>) -> anyhow::Result<bool> {
        log::info!("Process {:?}", msg);
        let msg_asr = msg.message;
        if msg.read_ct > 3 {
            log::warn!("Max retries reached");

            let mut external_id = "".to_string();
            if let Ok(item) = self.load_item(msg_asr.clone()).await {
                external_id = item.external_id;
            }

            // don't fail here, just try send status message
            if let Err(err) = self
                .send_status(&msg_asr, true, "max retries reached", &external_id)
                .await
            {
                log::error!("can't send status message: {}", err);
            }
            return Ok(true);
        }
        let mut item = self.load_item(msg_asr.clone()).await?;
        let ct = CancellationToken::new();
        let _st_dg = ct.clone().drop_guard();
        let job_handle: JoinHandle<()> = self.keep_in_progress(ct.clone(), msg.msg_id);

        if item.external_id.is_empty() {
            let external_id = self.upload(&msg_asr).await?;
            log::info!("Uploaded: {}", external_id);
            self.update_external_id(msg_asr.id.clone(), external_id.clone())
                .await?;
            item.external_id = external_id;
        }

        let (finished, err) = self.check_status(&item).await.map_err(anyhow::Error::msg)?;
        self.send_status(&msg_asr, finished, &err, &item.external_id)
            .await?;
        log::info!("finish: {}", msg.msg_id);
        log::debug!("sending cancel signal to update job...");
        ct.cancel();
        _ = job_handle.await;
        log::info!("done: {}", msg.msg_id);
        Ok(true)
    }

    async fn load_item(&self, msg_asr: ASRMessage) -> anyhow::Result<WorkData> {
        let conn = self.pool.get().await?;
        let result = conn
            .interact(move |conn| {
                use diesel::Connection;
                conn.transaction(|conn| {
                    use diesel::OptionalExtension;
                    use schema::work_data::dsl::*;
                    let res: Option<WorkData> = work_data
                        .filter(schema::work_data::id.eq(msg_asr.id.clone()))
                        .first(conn)
                        .optional()?;
                    if let Some(v) = res {
                        log::info!("Found: {}", v.id);
                        return Ok::<WorkData, anyhow::Error>(v);
                    }

                    let res: WorkData = diesel::insert_into(work_data)
                        .values((
                            id.eq(msg_asr.id.clone()),
                            file_name.eq(msg_asr.file.clone()),
                            base_dir.eq(msg_asr.base_dir.clone()),
                            external_id.eq(""),
                        ))
                        .get_result(conn)?;
                    log::info!("Inserted: {}", res.id);
                    Ok(res)
                })
            })
            .await
            .map_err(|err| format!("can't insert/get work data: {}", err))
            .map_err(anyhow::Error::msg)??;
        Ok(result)
    }

    async fn update_external_id(&self, id_v: String, external_id_v: String) -> anyhow::Result<()> {
        let conn = self.pool.get().await?;
        _ = conn
            .interact(move |conn| {
                use schema::work_data::dsl::*;
                diesel::update(work_data)
                    .filter(id.eq(id_v))
                    .set(external_id.eq(external_id_v))
                    .execute(conn)
            })
            .await
            .map_err(|err| format!("can't update work data: {}", err))
            .map_err(anyhow::Error::msg)??;
        Ok(())
    }

    fn keep_in_progress(&self, cl: CancellationToken, q_id: i64) -> JoinHandle<()> {
        let q = self.input_queue.clone();
        tokio::spawn(async move {
            log::info!("start loop...");
            loop {
                tokio::select! {
                    _ = sleep(Duration::from_secs(20)) => {}
                    _ = cl.cancelled() => {
                        log::debug!("job received cancel signal.");
                        break;
                    }
                }
                log::debug!("async job running...");
                if let Err(e) = q.mark_working(q_id).await {
                    log::error!("queue update error {}", e);
                }
            }
            log::info!("exit loop...");
        })
    }

    async fn upload(&self, msg_asr: &ASRMessage) -> anyhow::Result<String> {
        let file_path = format!("{}/working/{}", msg_asr.base_dir, msg_asr.file);
        self.asr_client.upload(file_path.as_str()).await
    }

    async fn get_status(
        &self,
        ext_id: &str,
    ) -> Result<(bool, String), Box<dyn Error + Send + Sync>> {
        let res = self.asr_client.status(ext_id).await?;
        log::info!("status: {:?}", res.status);
        if let Some(status) = res.status {
            if status == "COMPLETED" {
                return Ok((true, "".to_string()));
            }
        }
        if let Some(err_code) = res.error_code {
            return Ok((
                true,
                format!(
                    "{}\n{}",
                    err_code,
                    res.error.unwrap_or_else(|| "".to_string())
                )
                .to_string(),
            ));
        }
        Ok((false, "".to_string()))
    }

    async fn send_status(
        &self,
        orig: &ASRMessage,
        finished: bool,
        error: &str,
        external_id_v: &str,
    ) -> anyhow::Result<()> {
        log::info!(
            "send finished: {}, id: {}, err: {}",
            finished,
            orig.id,
            error
        );
        let status = ResultMessage {
            id: orig.id.clone(),
            file: orig.file.clone(),
            base_dir: orig.base_dir.clone(),
            finished,
            external_id: external_id_v.to_string(),
            error: if error.is_empty() {
                None
            } else {
                Some(error.to_string())
            },
        };
        self.result_queue.send(status).await
    }

    async fn check_status(
        &self,
        item: &WorkData,
    ) -> Result<(bool, String), Box<dyn Error + Send + Sync>> {
        let start_time = Instant::now();
        let wait_duration = Duration::from_secs(3600);
        log::info!("start check status: {} {}", item.id, item.external_id);
        let mut err_count = 0;
        loop {
            if start_time.elapsed() >= wait_duration {
                return Err("status wait timeout".into());
            }
            let jitter = {
                let mut rng = rand::thread_rng();
                Duration::from_millis(rng.gen_range(0..=5000))
            };
            tokio::select! {
                _ = sleep(Duration::from_secs(8) + jitter) => {}
                _ = self.ct.cancelled() => {
                    return Err("cancelled".into());
                }
            }
            let v = self.get_status(item.external_id.as_str()).await;
            match v {
                Ok(v) => {
                    err_count = 0;
                    if v.0 {
                        log::info!("completed");
                        return Ok(v);
                    }
                }
                Err(e) => {
                    err_count += 1;
                    log::error!("err {}: {}", err_count, e);
                    if err_count > 3 {
                        log::error!("max retries reached");
                        return Ok((false, e.to_string()));
                    }
                }
            }
        }
    }
}
