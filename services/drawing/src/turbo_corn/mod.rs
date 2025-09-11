mod draw;
mod job;

use anyhow::Result;
use chrono_tz::Tz;
use common::entity::{prelude::TurboTogelDrawShedule, turbo_togel_draw_shedule::Column};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::{str::FromStr, sync::Arc};
use tokio_cron_scheduler::{Job, JobScheduler};

pub struct Scheduler {
    db: Arc<sea_orm::DatabaseConnection>,
}

impl Scheduler {
    pub fn new(db: sea_orm::DatabaseConnection) -> Self {
        let db = Arc::new(db);

        Self { db }
    }

    pub async fn start(&self) -> Result<()> {
        let db = self.db.clone();

        // Find all active schedules
        let schedules = TurboTogelDrawShedule::find()
            .filter(Column::Status.eq(1))
            .all(&*self.db)
            .await?;

        let sched = JobScheduler::new().await?;

        // Schedule jobs for each active schedule
        for schedule in schedules {
            let timezone = Tz::from_str(&schedule.location)?;
            let db = db.clone();
            let job = Job::new_async_tz(schedule.cron.clone(), timezone, move |_uuid, _l| {
                Box::pin({
                    let db_clone = db.clone();
                    let schedule_clone = schedule.clone();
                    async move {
                        if let Err(e) = job::draw_turbo_togel(db_clone, schedule_clone).await {
                            tracing::error!("Draw job failed: {:?}", e);
                        }
                    }
                })
            })?;

            sched.add(job).await?;
        }

        sched.start().await?;

        Ok(())
    }
}
