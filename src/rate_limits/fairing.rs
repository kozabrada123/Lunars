use std::{net::IpAddr, str::FromStr};

use chrono::Utc;
use rocket::{
    fairing::{Fairing, Info, Kind},
    http::uri::Origin,
    request::FromRequest,
    Data, Orbit, Request, Response, Rocket,
};
use rocket_db_pools::Database;
use tokio::sync::Mutex;

use crate::{
    request_guards::api_key::ApiKey, types::entities::recent_request::RecentRequest, MysqlDb,
};

use super::{API_KEY_ALLOWED_REQUESTS_PER_MINUTE, UNAUTHENTICATED_ALLOWED_REQUESTS_PER_MINUTE};

#[derive(Default)]
pub struct RateLimiter {
    db: Mutex<Option<MysqlDb>>,
}

#[rocket::async_trait]
impl Fairing for RateLimiter {
    fn info(&self) -> Info {
        Info {
            name: "Ratelimiter",
            kind: Kind::Request | Kind::Response | Kind::Liftoff,
        }
    }

    async fn on_liftoff(&self, rocket: &Rocket<Orbit>) {
        let mut lock = self.db.lock().await;
        *lock = Some(MysqlDb::fetch(&rocket).unwrap().clone());
    }

    // Increment the counter for `GET` and `POST` requests.
    async fn on_request(&self, request: &mut Request<'_>, _data: &mut Data<'_>) {
        // Only ratelimit api requests, you can ddos the swagger ui if you'd like
        if !request.uri().to_string().starts_with("/api/") {
            return;
        }

        let db = self.db.lock().await.clone().unwrap();

        let now = Utc::now();
        let one_minute_ago = now - chrono::TimeDelta::minutes(1);

        // Check if it is authenticated or not
        let api_key_option = ApiKey::from_request(request).await.succeeded();

        if let Some(api_key) = api_key_option {
            let query_string = "SELECT * FROM recent_requests WHERE epoch > ? AND api_key_hash = ?";

            let query = sqlx::query_as(&query_string)
                .bind(one_minute_ago)
                .bind(api_key.hash);

            let requests_last_minute: Vec<RecentRequest> = query.fetch_all(&*db).await.unwrap();

            if requests_last_minute.len() >= API_KEY_ALLOWED_REQUESTS_PER_MINUTE {
                // No you dont!
                request.set_uri(Origin::parse("/.super_secret/you_are_rate_limited").unwrap());
            }
        } else {
            let ip = if let Some(header_ip) = request.headers().get_one("X-Real-IP") {
                IpAddr::from_str(header_ip).unwrap_or(request.remote().unwrap().ip())
            } else {
                request.remote().unwrap().ip()
            };

            let query_string = "SELECT * FROM recent_requests WHERE epoch > ? AND ip = ?";

            let query = sqlx::query_as(&query_string).bind(one_minute_ago).bind(ip);

            let requests_last_minute: Vec<RecentRequest> = query.fetch_all(&*db).await.unwrap();

            if requests_last_minute.len() >= UNAUTHENTICATED_ALLOWED_REQUESTS_PER_MINUTE {
                // No you dont!
                request.set_uri(Origin::parse("/.super_secret/you_are_rate_limited").unwrap());
            }
        }

        drop(db);
    }

    async fn on_response<'r>(&self, request: &'r Request<'_>, _response: &mut Response<'r>) {
        // Only ratelimit api requests, you can ddos the swagger ui if you'd like
        if !request.uri().to_string().starts_with("/api/") {
            return;
        }

        let db = self.db.lock().await.clone().unwrap();

        let now = Utc::now();

        // Check if it is authenticated or not
        let api_key_option = ApiKey::from_request(request).await.succeeded();

        if let Some(api_key) = api_key_option {
            let query_string = "INSERT INTO recent_requests (api_key_hash, epoch) VALUES (?, ?)";

            let query = sqlx::query(&query_string).bind(api_key.hash).bind(now);

            let res = query.execute(&*db).await;

            if let Err(e) = res {
                log::error!("Failed to add recent request: {} - {}", e, query_string);
            }
        } else {
            let ip = if let Some(header_ip) = request.headers().get_one("X-Real-IP") {
                IpAddr::from_str(header_ip).unwrap_or(request.remote().unwrap().ip())
            } else {
                request.remote().unwrap().ip()
            };

            let query_string = "INSERT INTO recent_requests (ip, epoch) VALUES (?, ?)";

            let query = sqlx::query(&query_string).bind(ip.to_string()).bind(now);

            let res = query.execute(&*db).await;

            if let Err(e) = res {
                log::error!("Failed to add recent request: {} - {}", e, query_string);
            }
        }

        let one_minute_ago = now - chrono::TimeDelta::minutes(1);

        let query_string = "DELETE FROM recent_requests WHERE epoch < ?";

        let query = sqlx::query(&query_string).bind(one_minute_ago);

        let res = query.execute(&*db).await;

        if let Err(e) = res {
            log::error!("Failed to clear recent requests: {} - {}", e, query_string);
        }

        drop(db);
    }
}
