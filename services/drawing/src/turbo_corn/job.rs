use anyhow::Result;
use chrono::{NaiveTime, TimeZone, Utc};

use common::entity::{prelude::TurboTogelDrawResult, turbo_togel_draw_result};
use sea_orm::{
    ActiveModelTrait, ActiveValue::Set, ColumnTrait, ConnectionTrait, EntityTrait, IntoActiveModel,
    QueryFilter, TransactionTrait,
};
use std::{str::FromStr, sync::Arc};

use crate::turbo_corn::distributed_lock::{DistributedLock, DistributedLockGuard};
use crate::turbo_corn::draw;

// Perform the draw operation for a given schedule with distributed lock
pub(super) async fn draw_turbo_togel_with_lock(
    db: Arc<sea_orm::DatabaseConnection>,
    schedule: common::entity::turbo_togel_draw_shedule::Model,
    redis_url: String,
) -> Result<()> {
    // Calculate period first (needed for lock key)
    let period = calculate_period(&schedule)?;

    // Create distributed lock
    let lock = DistributedLock::new(&redis_url)?;
    let lock_key = DistributedLock::generate_lock_key(schedule.id, &period);
    let lock_value = DistributedLock::generate_lock_value();

    // Store schedule ID for logging
    let schedule_id = schedule.id;

    // Try to acquire lock with 30 seconds TTL
    let lock_guard = DistributedLockGuard::try_acquire(
        lock, lock_key, lock_value, 30, // 30 seconds TTL
    )
    .await?;

    if let Some(_guard) = lock_guard {
        tracing::info!(
            "Acquired lock for schedule {} period {} - proceeding with draw",
            schedule_id,
            period
        );

        // We got the lock, proceed with the draw
        draw_turbo_togel(db, schedule).await?;

        tracing::info!(
            "Completed draw for schedule {} period {} - lock will be released automatically",
            schedule_id,
            period
        );
    } else {
        tracing::info!(
            "Failed to acquire lock for schedule {} period {} - skipping this execution (another pod is handling it)",
            schedule_id,
            period
        );
    }

    Ok(())
}

// Original draw function (now private)
async fn draw_turbo_togel(
    db: Arc<sea_orm::DatabaseConnection>,
    schedule: common::entity::turbo_togel_draw_shedule::Model,
) -> Result<()> {
    let mut saved_record = db
        .transaction::<_, turbo_togel_draw_result::Model, anyhow::Error>(|txn| {
            Box::pin(async move {
                // Calculate current period
                let period = calculate_period(&schedule)?;

                // Draw numbers
                let numbers = draw::draw_number(
                    schedule.min,
                    schedule.max,
                    schedule.count,
                    schedule.repeatable,
                )?;

                // Persist results and get the saved record
                let saved_record = save_draw_result(
                    txn,
                    schedule,
                    period,
                    numbers,
                    chrono::Utc::now().naive_utc(),
                )
                .await?;

                Ok(saved_record)
            })
        })
        .await?;

    // Publish to result to result

    // Change  the `is_broadcasted` to true
    db.transaction::<_, (), anyhow::Error>(|db| {
        Box::pin(async move {
            // Use UTC time for consistency across all records
            let current_time =
                chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());
            saved_record.is_broadcasted = true;
            saved_record.updated_at = Some(current_time);
            saved_record.into_active_model().update(&*db).await?;
            Ok(())
        })
    })
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
async fn save_draw_result<C: ConnectionTrait>(
    db: &C,
    schedule: common::entity::turbo_togel_draw_shedule::Model,
    period: String,
    numbers: String,
    _draw_time: chrono::NaiveDateTime,
) -> Result<turbo_togel_draw_result::Model> {
    tracing::info!(
        "Saving draw result to database - Schedule ID: {}, Period: {}, Result: {}",
        schedule.id,
        period,
        numbers
    );

    // Use UTC time for consistency across all records
    let current_time = chrono::Utc::now().with_timezone(&chrono::FixedOffset::east_opt(0).unwrap());

    // Try to insert first - let database handle uniqueness
    let new_result = turbo_togel_draw_result::ActiveModel {
        game_id: Set(schedule.id),
        period: Set(period.to_string()),
        numbers: Set(numbers),
        drawing_at: Set(Some(current_time)),
        created_at: Set(Some(current_time)),
        updated_at: Set(Some(current_time)),
        remark: Set(None),
        updated_by: Set(Some("system".to_string())),
        is_broadcasted: Set(false),
    };

    // Try to insert - if it fails due to unique constraint, fetch existing record
    match ActiveModelTrait::insert(new_result, db).await {
        Ok(inserted_record) => {
            tracing::info!(
                "Successfully inserted new record for GameID: {}, Period: {}",
                schedule.id,
                period
            );
            Ok(inserted_record)
        }
        Err(sea_orm::DbErr::Exec(_)) => {
            // Database execution error - likely unique constraint violation
            tracing::info!(
                "Insert failed (likely unique constraint) - fetching existing for GameID: {}, Period: {}",
                schedule.id,
                period
            );

            // Fetch the existing record
            let existing_record = TurboTogelDrawResult::find()
                .filter(turbo_togel_draw_result::Column::GameId.eq(schedule.id))
                .filter(turbo_togel_draw_result::Column::Period.eq(&period))
                .one(db)
                .await?
                .ok_or_else(|| anyhow::anyhow!("Record should exist but not found"))?;

            Ok(existing_record)
        }
        Err(e) => {
            tracing::error!("Failed to insert draw result: {:?}", e);
            Err(e.into())
        }
    }
}
