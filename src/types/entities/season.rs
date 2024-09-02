use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlRow, FromRow, Row};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd, JsonSchema)]
/// A rating period of the ranking system;
///
/// After the end of this span of time, players' ranks will be
/// calculated and commited to the database
pub struct Season {
    /// Sequential id of the rating period
    pub id: u64,
    /// When the rating period started, Utc time
    pub start: DateTime<Utc>,
    /// When the rating period ended or will end, Utc time
    pub end: DateTime<Utc>,
    /// Whether or not the ranked data from this season
    /// has been processed and written to the database yet
    pub processed: bool,
}

impl<'r> FromRow<'r, MySqlRow> for Season {
    fn from_row(row: &'r MySqlRow) -> Result<Self, sqlx::Error> {
        let id = row.try_get("id")?;

        let start = row.try_get("start")?;

        let end = row.try_get("end")?;

        let processed = row.try_get("processed")?;

        Ok(Season {
            id,
            start,
            end,
            processed,
        })
    }
}

impl Season {
    /// Returns whether we are within the time bounds of the rating period
    pub fn is_active(&self) -> bool {
        let current_time = Utc::now();

        current_time >= self.start && current_time < self.end
    }

    pub fn new(start: DateTime<Utc>, end: DateTime<Utc>) -> Self {
        Self {
            start,
            end,
            id: 0,
            processed: false,
        }
    }

    /// Creates a new rating period starting now and ending at the given end
    pub fn new_starting_now_until(end: DateTime<Utc>) -> Self {
        let now = Utc::now();

        Self {
            start: now,
            end,
            id: 0,
            processed: false,
        }
    }

    /// Creates a new rating period starting now and ending after the duration
    pub fn from_duration(duration: chrono::Duration) -> Self {
        let now = Utc::now();

        let end = now + duration;

        Self {
            start: now,
            end,
            id: 0,
            processed: false,
        }
    }

    /// Returns the duration of the rating period
    pub fn duration(&self) -> chrono::Duration {
        self.end - self.start
    }

    /// Returns how much time has elapsed since the start of the season
    pub fn elapsed_since_start(&self) -> chrono::Duration {
        let now = Utc::now();

        now - self.start
    }

    /// Returns how much time has elapsed since the end of the season
    pub fn elapsed_since_end(&self) -> chrono::Duration {
        let now = Utc::now();

        now - self.end
    }

    /// Returns how many times the season has completed over.
    ///
    /// At self.start, this returns 0.0
    /// At self.end, this returns 1.0
    pub fn completion(&self) -> f64 {
        let since_start = self.elapsed_since_start();

        let duration = self.duration();

        since_start.num_milliseconds() as f64 / duration.num_milliseconds() as f64
    }
}
