use core::panic;

use chrono::Utc;
use sqlx::mysql::MySqlQueryResult;

use crate::types::entities::season::Season;

use super::{query::QueryParameters, DbConnection};

impl DbConnection {
    /// Fetches all the rating_periods.
    ///
    /// Use query_parameters to set order_by, max, min, ...
    pub async fn get_seasons(&mut self, query_parameters: QueryParameters) -> Vec<Season> {
        let query_string = "SELECT * FROM rating_periods";

        let (query_string, parameters) = self.add_to_query(query_string, query_parameters).await;

        let mut query = sqlx::query_as(query_string.as_str());

        for parameter in parameters {
            query = query.bind(parameter);
        }

        let result = query.fetch_all(&mut **self.inner).await;

        match result {
            Ok(vec) => return vec,
            Err(e) => match e {
                sqlx::Error::RowNotFound => return Vec::new(),
                _ => {
                    log::error!("Database query failed {} -> {}", query_string, e);
                    panic!("Database query failed");
                }
            },
        }
    }

    /// Fetches a rating period by id
    pub async fn get_season_by_id(&mut self, id: u64) -> Option<Season> {
        let query_string = "SELECT * FROM rating_periods WHERE id = ?";

        let query = sqlx::query_as(&query_string).bind(id);

        let result: Result<Season, sqlx::Error> = query.fetch_one(&mut **self.inner).await;

        match result {
            Ok(season) => {
                return Some(season);
            }
            Err(e) => match e {
                sqlx::Error::RowNotFound => return None,
                _ => {
                    log::error!("Database query failed {} -> {}", query_string, e);
                    panic!("Database query failed");
                }
            },
        }
    }

    /// Fetches the latest active rating period.
    pub async fn get_latest_active_season(&mut self) -> Option<Season> {
        let now = Utc::now();

        let query_string =
            "SELECT * FROM rating_periods WHERE (start < ? AND end > ?) ORDER BY id DESC LIMIT 1";

        let query = sqlx::query_as(&query_string).bind(now).bind(now);

        let result: Result<Season, sqlx::Error> = query.fetch_one(&mut **self.inner).await;

        match result {
            Ok(season) => {
                return Some(season);
            }
            Err(e) => match e {
                sqlx::Error::RowNotFound => return None,
                _ => {
                    log::error!("Database query failed {} -> {}", query_string, e);
                    panic!("Database query failed");
                }
            },
        }
    }

    /// Updates a rating period.
    ///
    /// Every field can be changed except id.
    pub async fn modify_season(
        &mut self,
        season: &Season,
    ) -> Result<MySqlQueryResult, sqlx::Error> {
        let query_string =
            "UPDATE rating_periods SET start = ?, end = ?, processed = ? WHERE id = ?";

        let query = sqlx::query(&query_string)
            .bind(season.start)
            .bind(season.end)
            .bind(season.processed)
            .bind(season.id);

        let result = query.execute(&mut **self.inner).await;

        match result {
            Ok(result) => {
                return Ok(result);
            }
            Err(e) => match e {
                _ => {
                    log::error!("Database query failed {} -> {}", query_string, e);
                    panic!("Database query failed");
                }
            },
        }
    }

    /// Adds a rating period.
    ///
    /// Ignores the id field.
    pub async fn add_rating_period(
        &mut self,
        season: &Season,
    ) -> Result<MySqlQueryResult, sqlx::Error> {
        let query_string = "INSERT INTO rating_periods (start, end, processed) VALUES (?, ?)";

        let query = sqlx::query(&query_string)
            .bind(season.start)
            .bind(season.end)
            .bind(season.processed);

        let result = query.execute(&mut **self.inner).await;

        match result {
            Ok(result) => {
                return Ok(result);
            }
            Err(e) => match e {
                _ => {
                    log::error!("Database query failed {} -> {}", query_string, e);
                    panic!("Database query failed");
                }
            },
        }
    }
}
