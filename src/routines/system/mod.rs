use actix_web::rt::time;
use chrono::{Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike, Utc};
use diesel::prelude::*;

use crate::db::get_pg_connection;
use crate::db::models::api_config::ApiConfig;
use crate::db::models::maintenance::Maintenance;
use crate::routines::market::perform_maintenance;
use crate::structs::api_config::LockedApiConfig;

pub async fn poll_api_online_status(config_lock: LockedApiConfig) {
    use crate::db::schema::api_config as schema;
    use crate::db::schema::maintenance as m_schema;

    let mut interval = time::interval(core::time::Duration::from_secs(60));
    loop {
        interval.tick().await;
        let utc_now = Utc::now().naive_utc();
        let mut new_config;
        let maintenance_mode;

        // Remember current config version
        let current_config = (*config_lock.read()).clone();
        if current_config.is_none() {
            warn!("Configuration hasn't been initialised yet!");
            break;
        }
        let current_config = current_config.unwrap();
        let db_api_config: Option<ApiConfig>;
        let maintenance_data: Option<Maintenance>;

        // Ask the database for latest config, wrap in scope to prevent wait across boundaries
        {
            debug!("Checking API status");
            let conn = &get_pg_connection();
            db_api_config = match schema::table
                .order_by(schema::version.desc())
                .first::<ApiConfig>(conn)
            {
                Ok(row) => Some(row),
                _ => None,
            };

            // Did we get a new config from the database?
            if let Some(db_api_config) = db_api_config {
                debug!("DB Config Version: {:?}", db_api_config.version);
                if db_api_config.version > current_config.version {
                    info!(
                        "Applying new config, Version {} > {}",
                        db_api_config.version, current_config.version
                    );
                    // Set ptr -> new config
                    new_config = db_api_config
                } else {
                    // Just use the current config
                    new_config = current_config
                }
            } else {
                debug!("Status: No answer from database or config not found, using defaults");
                // Create a new config and set ptr
                new_config = ApiConfig::default()
            }

            debug!("Checking API Maintenance status");
            maintenance_data = match m_schema::table.first::<Maintenance>(conn) {
                Ok(row) => Some(row),
                _ => None,
            };
        } // Drop DB Scope

        maintenance_mode = if let Some(maintenance_data) = &maintenance_data {
            debug!("Maintenance data {:?}", maintenance_data);
            maintenance_data.in_progress
        } else {
            debug!("Status: No answer from database or maintenance data not found, using defaults");
            false
        };

        if maintenance_mode {
            // Force the API offline if we're in maintenance mode
            new_config.config_data.api.force_offline = true
        }

        {
            // Get a write lock and update the config in memory
            let mut config = config_lock.write();
            *config = Some(new_config.clone()); // Update the pointer to the new config struct
                                                // Drop the write lock
            drop(config);
        }

        debug!("Status: {:?}", new_config);

        // If we're:
        // * Not currently in maintenance mode
        // * AND it's after the scheduled start time
        // * AND it hasn't already run today
        // * OR there was no previous maintenance entry
        // Then run maintenance.

        if !maintenance_mode
            && is_now_after(new_config.config_data.maintenance.start_time)
            && maintenance_data.as_ref().map_or_else(
                || true,
                |m| {
                    m.execution_time
                        .map_or(true, |ed| ed.date() < utc_now.date())
                },
            )
            || maintenance_data.as_ref().is_none()
        {
            // Do maintenance
            perform_maintenance(&new_config).await;
        } else {
            let next_run_day;
            let last_run = maintenance_data
                .unwrap_or_default()
                .execution_time
                .unwrap_or(NaiveDateTime::from_timestamp(0, 0));
            // If we're still on the same day, add 1 day, otherwise we're already ahead.
            if utc_now.date() == last_run.date() {
                next_run_day = date_midnight(utc_now.date() + Duration::days(1));
            } else {
                next_run_day = date_midnight(utc_now.date())
            }
            let next_run = next_run_day
                + Duration::seconds(new_config.config_data.maintenance.start_time as i64);
            debug!("Time until next maintenance: {}", (next_run - utc_now))
        }
    }
}

pub fn utc_midnight() -> NaiveDateTime {
    date_midnight(Utc::now().naive_utc().date())
}

pub fn date_midnight(date: NaiveDate) -> NaiveDateTime {
    date.and_time(NaiveTime::from_hms(0, 0, 0))
}

pub fn is_now_after(time: u32) -> bool {
    let now = Utc::now().num_seconds_from_midnight();
    now > time
}
