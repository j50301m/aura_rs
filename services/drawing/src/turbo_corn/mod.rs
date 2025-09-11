mod draw;
mod job;

use anyhow::Result;
use chrono_tz::Tz;
use common::{
    cache,
    entity::{prelude::TurboTogelDrawShedule, turbo_togel_draw_shedule::Column},
};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::{str::FromStr, sync::Arc};
use tokio_cron_scheduler::{Job, JobScheduler};

pub struct Scheduler {
    db: Arc<sea_orm::DatabaseConnection>,
    cache: Arc<cache::Cache>,
}

impl Scheduler {
    pub fn new(db: Arc<sea_orm::DatabaseConnection>, cache: Arc<cache::Cache>) -> Self {
        Self { db, cache }
    }

    pub async fn start(&self) -> Result<()> {
        // let db = self.db.clone();

        // Find all active schedules
        let schedules = TurboTogelDrawShedule::find()
            .filter(Column::Status.eq(1))
            .all(&*self.db)
            .await?;

        let sched = JobScheduler::new().await?;

        // Schedule jobs for each active schedule
        for schedule in schedules {
            let timezone = Tz::from_str(&schedule.location)?;
            let cache = self.cache.clone();
            let db = self.db.clone();
            let job = Job::new_async_tz(schedule.cron.clone(), timezone, move |_uuid, _l| {
                Box::pin({
                    let db_clone = db.clone();
                    let schedule_clone = schedule.clone();
                    let cache_clone = cache.clone();
                    async move {
                        job::draw_turbo_togel(cache_clone, db_clone, schedule_clone)
                            .await
                            .unwrap_or_else(|e| {
                                tracing::error!("Draw job failed: {:?}", e);
                            });
                    }
                })
            })?;

            sched.add(job).await?;
        }

        sched.start().await?;

        Ok(())
    }
}
