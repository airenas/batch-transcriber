use std::time::Instant;
use std::{error::Error, time::Duration};

use deadpool_diesel::postgres::Pool;
use diesel::query_dsl::methods::FilterDsl;
use diesel::ExpressionMethods;
use diesel::RunQueryDsl;
use pgmq::Message;
use tokio::{select, task::JoinHandle, time::sleep};
use tokio_util::sync::CancellationToken;
use rand::Rng;

use crate::{
    data::api::ASRMessage,
    filer::file::Filer,
    model::{
        models::WorkData,
        schema::{self},
    },
    postgres::queue::PQueue,
};

use super::client::ASRClient;

pub struct Worker {
    pgmq: PQueue,
    // filer: Filer,
    id: i32,
    ct: CancellationToken,
    pool: Pool,
    asr_client: ASRClient,
}

impl Worker {
    pub async fn new(
        pgmq: PQueue,
        _filer: Filer,
        id: i32,
        ct: CancellationToken,
        pool: Pool,
        asr_client: ASRClient,
    ) -> Result<Self, Box<dyn Error>> {
        log::info!("Init Worker");
        Ok(Self {
            pgmq,
            // filer,
            id,
            ct,
            pool,
            asr_client,
        })
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error + Send + Sync>> {
        log::info!("Run worker {}", self.id);
        loop {
            let mut was: bool = false;
            let res = self
                .pgmq
                .process(|msg: Message<ASRMessage>| async move { self.process_msg(msg).await })
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
                log::info!("Worker {} cancelled", self.id);
                break;
            }
            if !was {
                select! {
                    _ = self.ct.cancelled() => {
                        log::info!("Worker {} cancelled", self.id);
                        break;
                    }
                    _ = sleep(Duration::from_secs(1)) => { }
                }
            }
        }
        log::info!("Stop worker: {}", self.id);
        Ok(())
    }

    pub async fn process_msg(
        &self,
        msg: Message<ASRMessage>,
    ) -> Result<bool, Box<dyn Error + Send + Sync>> {
        log::info!("Process {:?}", msg);
        if msg.read_ct > 3 {
            log::warn!("Max retries reached");
            self.send_status(false, "max retries reached").await?;
            return Ok(true);
        }
        let msg_asr = msg.message;
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

        let (finished, err) = self.check_status(&item).await?;
        self.send_status(finished, err.as_str()).await?;
        log::info!("finish: {}", msg.msg_id);
        log::debug!("sending cancel signal to update job...");
        ct.cancel();
        _ = job_handle.await;
        log::info!("done: {}", msg.msg_id);
        Ok(true)
    }

    async fn load_item(
        &self,
        msg_asr: ASRMessage,
    ) -> Result<WorkData, Box<dyn Error + Send + Sync>> {
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
                        return Ok::<WorkData, Box<dyn Error + Send + Sync>>(v);
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
            .map_err(|err| format!("can't insert/get work data: {}", err))??;
        Ok(result)
    }

    async fn update_external_id(
        &self,
        id_v: String,
        external_id_v: String,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
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
            .map_err(|err| format!("can't update work data: {}", err))??;
        Ok(())
    }

    fn keep_in_progress(&self, cl: CancellationToken, q_id: i64) -> JoinHandle<()> {
        let q = self.pgmq.clone();
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

    async fn upload(&self, msg_asr: &ASRMessage) -> Result<String, Box<dyn Error + Send + Sync>> {
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
            return Ok((true, format!("{}\n{}", err_code, res.error.unwrap_or_else(||"".to_string())).to_string()));
        }
        Ok((false, "".to_string()))
    }

    async fn send_status(&self, ok: bool, error: &str) -> Result<(), Box<dyn Error + Send + Sync>> {
        log::info!("send status: {} {}", ok, error);
        Ok(())
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
