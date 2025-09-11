mod draw;

use anyhow::Result;
use chrono::{NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use common::entity::{prelude::TurboTogelDrawShedule, turbo_togel_draw_shedule::Column};
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};
use std::str::FromStr;
use tokio_cron_scheduler::{Job, JobScheduler};
use tracing::error;

pub struct Scheduler {
    db: sea_orm::DatabaseConnection,
}

impl Scheduler {
    pub fn new(db: sea_orm::DatabaseConnection) -> Self {
        Self { db }
    }

    pub async fn start(&self) -> Result<()> {
        // Find all active schedules
        let schedules = TurboTogelDrawShedule::find()
            .filter(Column::Status.eq(1))
            .all(&self.db)
            .await?;

        let sched = JobScheduler::new().await?;

        for schedule in schedules.into_iter() {
            let timezone = Tz::from_str(&schedule.location)?;

            let schedule_clone = schedule.clone();
            let job = Job::new_tz(schedule.cron.clone(), timezone, move |_uuid, _l| {
                // Calculate current period
                let Ok(period) = Self::calculate_current_period(&schedule_clone) else {
                    error!(
                        "Failed to calculate current period for schedule ID {}",
                        schedule_clone.id
                    );
                    return;
                };

                // Draw numbers
                let Ok(draw_result) = draw::draw_number(
                    schedule_clone.min,
                    schedule_clone.max,
                    schedule_clone.count,
                    schedule_clone.repeatable,
                ) else {
                    error!(
                        "Failed to draw numbers for schedule ID {}",
                        schedule_clone.id
                    );
                    return;
                };

                // Write into database

                println!("Draw result: {} for period: {}", draw_result, period);
            })?;

            sched.add(job).await?;
        }
        sched.start().await?;

        Ok(())
    }

    // 計算period
    fn calculate_current_period(
        entity: &common::entity::turbo_togel_draw_shedule::Model,
    ) -> Result<String> {
        // 解析時區
        let tz = chrono_tz::Tz::from_str(&entity.location)?;

        // 解析 first_draw 時間 (格式: "HH:MM:SS")
        let first_draw_time = NaiveTime::parse_from_str(&entity.first_draw, "%H:%M:%S")?;

        // 取得當前時區的當前時間
        let now_in_tz = Utc::now().with_timezone(&tz);
        let today = now_in_tz.date_naive();

        // 組合今天的日期和 first_draw 時間，創建時區感知的 DateTime
        let first_draw_today = tz
            .from_local_datetime(&today.and_time(first_draw_time))
            .single()
            .ok_or_else(|| anyhow::anyhow!("無法解析 first_draw 時間到指定時區"))?;

        // Parse
        let interval = entity.interval; // in seconds

        let period = ((now_in_tz.timestamp() - first_draw_today.timestamp()) / interval as i64) + 1;
        let period = format!("{}{:04}", today.format("%Y%m%d"), period);

        Ok(period)
    }
}
