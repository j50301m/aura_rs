mod draw;

use anyhow::Result;
use chrono::{NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use common::entity::{
    prelude::{TurboTogelDrawResult, TurboTogelDrawShedule},
    turbo_togel_draw_result,
    turbo_togel_draw_shedule::Column,
};
use rand::rand_core::le;
use sea_orm::{ActiveModelTrait, ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter};
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

        for schedule in schedules {
            let timezone = Tz::from_str(&schedule.location)?;
            let db = db.clone();
            let job = Job::new_async_tz(schedule.cron.clone(), timezone, move |_uuid, _l| {
                Box::pin({
                    let db_clone = db.clone();
                    let schedule_clone = schedule.clone();
                    async move {
                        if let Err(e) = draw_turbo_togel(db_clone, schedule_clone).await {
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

async fn draw_turbo_togel(
    db: Arc<sea_orm::DatabaseConnection>,
    schedule: common::entity::turbo_togel_draw_shedule::Model,
) -> Result<()> {
    // Calculate current period
    let period = calculate_period(&schedule)?;

    // Draw numbers
    let numbers = draw::draw_number(
        schedule.min,
        schedule.max,
        schedule.count,
        schedule.repeatable,
    )?;

    // Persist results
    save_draw_result(
        &db,
        schedule,
        period,
        numbers,
        chrono::Utc::now().naive_utc(),
    )
    .await?;

    Ok(())
}

// Calculate current period
fn calculate_period(entity: &common::entity::turbo_togel_draw_shedule::Model) -> Result<String> {
    // Parse timezone
    let tz = chrono_tz::Tz::from_str(&entity.location)?;

    // Parse first_draw time (format: "HH:MM:SS")
    let first_draw_time = NaiveTime::parse_from_str(&entity.first_draw, "%H:%M:%S")?;

    // Get current time in the specified timezone
    let now_in_tz = Utc::now().with_timezone(&tz);
    let today = now_in_tz.date_naive();

    // Combine today's date with first_draw time to create timezone-aware DateTime
    let first_draw_today = tz
        .from_local_datetime(&today.and_time(first_draw_time))
        .single()
        .ok_or_else(|| anyhow::anyhow!("Unable to parse first_draw time to specified timezone"))?;

    // Parse
    let interval = entity.interval; // in seconds

    let period = ((now_in_tz.timestamp() - first_draw_today.timestamp()) / interval as i64) + 1;
    let period = format!("{}{:04}", today.format("%Y%m%d"), period);

    Ok(period)
}

// Save draw results to database
async fn save_draw_result(
    db: &sea_orm::DatabaseConnection,
    schedule: common::entity::turbo_togel_draw_shedule::Model,
    period: String,
    numbers: String,
    _draw_time: chrono::NaiveDateTime,
) -> Result<()> {
    tracing::info!(
        "Saving draw result to database - Schedule ID: {}, Period: {}, Result: {}",
        schedule.id,
        period,
        numbers
    );

    // First, check if record exists and has "force tool" remark
    let existing_record = TurboTogelDrawResult::find()
        .filter(turbo_togel_draw_result::Column::GameId.eq(schedule.id))
        .filter(turbo_togel_draw_result::Column::Period.eq(&period))
        .one(db)
        .await?;

    if let Some(record) = existing_record {
        // Check if remark is "force tool"
        if let Some(ref remark) = record.remark {
            if remark == "force tool" {
                tracing::info!(
                    "Record exists with 'force tool' remark - doing nothing for GameID: {}, Period: {}",
                    schedule.id,
                    period
                );
                return Ok(());
            }
        }

        // Record exists but remark is not "force tool" - update it
        tracing::info!(
            "Updating existing record for GameID: {}, Period: {}",
            schedule.id,
            period
        );

        let mut active_model: turbo_togel_draw_result::ActiveModel = record.into();
        active_model.numbers = Set(numbers);
        active_model.drawing_at = Set(Some(
            chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap()),
        ));
        active_model.updated_at = Set(Some(
            chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap()),
        ));
        active_model.updated_by = Set(Some("system".to_string()));
        active_model.is_broadcasted = Set(false);

        ActiveModelTrait::update(active_model, db).await?;
    } else {
        // Record doesn't exist - insert new one
        tracing::info!(
            "Inserting new record for GameID: {}, Period: {}",
            schedule.id,
            period
        );

        let new_result = turbo_togel_draw_result::ActiveModel {
            game_id: Set(schedule.id),
            period: Set(period.to_string()),
            numbers: Set(numbers),
            drawing_at: Set(Some(
                chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap()),
            )),
            created_at: Set(Some(
                chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap()),
            )),
            updated_at: Set(Some(
                chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap()),
            )),
            remark: Set(None),
            updated_by: Set(Some("system".to_string())),
            is_broadcasted: Set(false),
            ..Default::default()
        };

        ActiveModelTrait::insert(new_result, db).await?;
    }

    Ok(())
}
