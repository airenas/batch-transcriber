use std::{error::Error, time::Duration};

use deadpool_diesel::postgres::Pool;
use diesel::{RunQueryDsl, SelectableHelper};
use pgmq::Message;
use tokio::{select, time::sleep};
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
    id: i64,
    ct: CancellationToken,
    pool: Pool,
}

impl Worker {
    pub async fn new(
        pgmq: PQueue,
        _filer: Filer,
        id: i64,
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

        let msg = msg.message;

        let work_data = WorkData {
            id: msg.id.clone(),
            external_id: "".to_string(),
            file_name: msg.file.clone(),
            base_dir: msg.base_dir.clone(),
            try_count: 0,
            created: chrono::Utc::now().naive_utc(),
            updated: chrono::Utc::now().naive_utc(),
        };
        let conn = self.pool.get().await?;

        let result = conn
            .interact(move |conn| {
                diesel::insert_into(schema::work_data::table)
                    .values(&work_data)
                    .returning(WorkData::as_returning())
                    .execute(conn)
            })
            .await
            .map_err(|err| format!("can't insert: {}", err))??;
        log::info!("Inserted: {:?}", result);
        Err("some err".into())
        // Ok(true)
    }
}
