use std::{error::Error, time::Duration};

use deadpool_diesel::postgres::Pool;
use diesel::query_dsl::methods::FilterDsl;
use diesel::ExpressionMethods;
use diesel::{RunQueryDsl, SelectableHelper};
use pgmq::Message;
use tokio::{select, task::JoinHandle, time::sleep};
use tokio_util::sync::CancellationToken;

use crate::{
    data::api::ASRMessage,
    filer::file::Filer,
    model::{
        models::WorkData,
        schema::{self},
    },
    postgres::queue::PQueue,
};

pub struct Worker {
    pgmq: PQueue,
    // filer: Filer,
    id: i32,
    ct: CancellationToken,
    pool: Pool,
}

impl Worker {
    pub async fn new(
        pgmq: PQueue,
        _filer: Filer,
        id: i32,
        ct: CancellationToken,
        pool: Pool,
    ) -> Result<Self, Box<dyn Error>> {
        log::info!("Init Worker");
        Ok(Self {
            pgmq,
            // filer,
            id,
            ct,
            pool,
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
        // sleep(Duration::from_secs(5)).await;
        if msg.read_ct > 3 {
            log::warn!("Max retries reached");
            return Ok(true);
        }

        let msg_asr = msg.message;

        let item = WorkData {
            id: msg_asr.id.clone(),
            external_id: "".to_string(),
            file_name: msg_asr.file.clone(),
            base_dir: msg_asr.base_dir.clone(),
            try_count: 0,
            created: chrono::Utc::now().naive_utc(),
            updated: chrono::Utc::now().naive_utc(),
        };
        let conn = self.pool.get().await?;

        let result = conn
            .interact(move |conn| {
                use diesel::Connection;
                conn.transaction(|conn| {
                    use self::schema::work_data::dsl::*;
                    use diesel::OptionalExtension;
                    let res: Option<WorkData> = work_data
                        .filter(schema::work_data::id.eq(&item.id))
                        .first(conn)
                        .optional()?;
                    if let Some(v) = res {
                        log::info!("Already exists: {:?}", v.id);
                        return Ok::<WorkData, Box<dyn Error + Send + Sync>>(v);
                    }

                    _ = diesel::insert_into(schema::work_data::table)
                        .values(&item)
                        .returning(WorkData::as_returning())
                        .execute(conn)?;
                    Ok(item)
                })
            })
            .await
            .map_err(|err| format!("can't insert: {}", err))??;
        log::info!("Inserted: {:?}", result.id);
        let token = CancellationToken::new();
        let token_cl = token.clone();
        let q = self.pgmq.clone();
        let id = msg.msg_id;
        let job_handle: JoinHandle<()> = tokio::spawn(async move {
            log::info!("Start loop...");
            loop {
                tokio::select! {
                    _ = sleep(Duration::from_secs(20)) => {}
                    _ = token_cl.cancelled() => {
                        log::info!("Job received cancel signal.");
                        break;
                    }
                }
                log::info!("Async job running...");
                if let Err(e) = q.mark_working(id).await {
                    log::error!("{}", e);
                }
            }
            log::info!("exit loop...");
        });
        log::info!("Wait for complete");
        tokio::select! {
            _ = sleep(Duration::from_secs(180)) => {}
            _ = self.ct.cancelled() => {
                log::info!("Worker {} cancelled", self.id);
                return Ok(false);
             }
        }
        log::info!("get result: {id}");

        log::info!("finish: {id}");
        log::info!("Sending cancel signal to the job...");
        token.cancel();
        let _ = job_handle.await;
        log::info!("done: {id}");
        Ok(true)
    }
}
