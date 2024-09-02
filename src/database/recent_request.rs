use core::panic;
use std::net::IpAddr;

use chrono::Utc;
use sqlx::mysql::MySqlQueryResult;

use crate::types::entities::recent_request::RecentRequest;

use super::DbConnection;

impl DbConnection {
    /// Removes all non-recent requests (ones that are > 1 minute old)
    pub async fn remove_non_recent_requests(&mut self) -> Result<MySqlQueryResult, sqlx::Error> {
        let now = Utc::now();
        let one_minute_ago = now - chrono::TimeDelta::minutes(1);

        let query_string = "DELETE FROM recent_requests WHERE epoch < ?";

        let query = sqlx::query(&query_string).bind(one_minute_ago);

        query.execute(&mut **self.inner).await
    }

    /// Fetches all recent requests for an ip, which occured in the last minute.
    pub async fn get_recent_requests_for_ip(&mut self, ip: IpAddr) -> Vec<RecentRequest> {
        let now = Utc::now();
        let one_minute_ago = now - chrono::TimeDelta::minutes(1);

        let query_string = "SELECT * FROM recent_requests WHERE epoch > ? AND ip = ?";

        let query = sqlx::query_as(&query_string)
            .bind(one_minute_ago)
            .bind(ip.to_string());

        let result: Result<Vec<RecentRequest>, sqlx::Error> =
            query.fetch_all(&mut **self.inner).await;

        match result {
            Ok(requests) => {
                return requests;
            }
            Err(e) => match e {
                sqlx::Error::RowNotFound => return Vec::new(),
                _ => {
                    log::error!("Database query failed {} -> {}", query_string, e);
                    panic!("Database query failed");
                }
            },
        }
    }

    /// Fetches all recent requests for an api key hash, which occured in the last minute.
    pub async fn get_recent_requests_for_api_key_hash(
        &mut self,
        api_key_hash: String,
    ) -> Vec<RecentRequest> {
        let now = Utc::now();
        let one_minute_ago = now - chrono::TimeDelta::minutes(1);

        let query_string = "SELECT * FROM recent_requests WHERE epoch > ? AND api_key_hash = ?";

        let query = sqlx::query_as(&query_string)
            .bind(one_minute_ago)
            .bind(api_key_hash.to_string());

        let result: Result<Vec<RecentRequest>, sqlx::Error> =
            query.fetch_all(&mut **self.inner).await;

        match result {
            Ok(requests) => {
                return requests;
            }
            Err(e) => match e {
                sqlx::Error::RowNotFound => return Vec::new(),
                _ => {
                    log::error!("Database query failed {} -> {}", query_string, e);
                    panic!("Database query failed");
                }
            },
        }
    }

    /// Adds a recent request.
    pub async fn add_recent_request(
        &mut self,
        request: RecentRequest,
    ) -> Result<MySqlQueryResult, sqlx::Error> {
        let query_string = "INSERT INTO recent_requests (ip, api_key_hash, epoch) VALUES (?, ?, ?)";

        let query = sqlx::query(&query_string)
            .bind(request.ip)
            .bind(request.api_key_hash)
            .bind(request.epoch);

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
