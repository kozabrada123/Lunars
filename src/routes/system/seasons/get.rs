use rocket::{get, http::Status, serde::json::Json};
use rocket_db_pools::Connection;
use rocket_okapi::openapi;

use crate::{
    database::{query::QueryParameters, DbConnection},
    request_guards::chrono::chrono_timestamp_from_string,
    response::ApiError,
    types::entities::season::Season,
    MysqlDb,
};

#[openapi(ignore = "db", tag = "System")]
#[get(
    "/system/seasons?<start_before>&<start_after>&<end_before>&<end_after>&<sort>&<limit>&<offset>"
)]
/// Fetches an array of all rating periods.
///
/// Here ?start_before, ?start_after, ?end_before and ?end_after can be used to target the start and end point of periods (in Utc time)
///
/// They can be set to either an rfc3339 (iso) timestamp or unix milliseconds
pub async fn get_seasons(
    db: Connection<MysqlDb>,
    start_after: Option<String>,
    start_before: Option<String>,
    end_after: Option<String>,
    end_before: Option<String>,
    sort: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Json<Vec<Season>> {
    let mut start_after_chrono = None;
    let mut start_before_chrono = None;
    let mut end_after_chrono = None;
    let mut end_before_chrono = None;

    if let Some(end_after_timestamp) = end_after {
        end_after_chrono = chrono_timestamp_from_string(&end_after_timestamp);
    }

    if let Some(end_before_timestamp) = end_before {
        end_before_chrono = chrono_timestamp_from_string(&end_before_timestamp);
    }

    if let Some(start_after_timestamp) = start_after {
        start_after_chrono = chrono_timestamp_from_string(&start_after_timestamp);
    }

    if let Some(start_before_timestamp) = start_before {
        start_before_chrono = chrono_timestamp_from_string(&start_before_timestamp);
    }

    let query_parameters = QueryParameters {
        end_after: end_after_chrono,
        end_before: end_before_chrono,
        start_after: start_after_chrono,
        start_before: start_before_chrono,
        sort,
        limit,
        offset,
        ..Default::default()
    };

    let mut database_connection = DbConnection::from_inner(db);

    Json(database_connection.get_seasons(query_parameters).await)
}

#[openapi(ignore = "db", tag = "System")]
#[get("/system/seasons/<id>")]
/// Fetches a rating period via its id.
///
/// If no such match is found, the [ApiError] will have code 0 and message "Not Found"
pub async fn get_season(db: Connection<MysqlDb>, id: u64) -> Result<Json<Season>, ApiError> {
    let mut database_connection = DbConnection::from_inner(db);

    let season_option = database_connection.get_season_by_id(id).await;

    match season_option {
        None => Err(ApiError::from_status(Status::NotFound)),
        Some(season) => Ok(Json(season)),
    }
}

#[openapi(ignore = "db", tag = "System")]
#[get("/system/seasons/latest")]
/// Fetches the latest rating period.
///
/// If no such match is found, the [ApiError] will have code 0 and message "Not Found"
pub async fn get_latest_season(db: Connection<MysqlDb>) -> Result<Json<Season>, ApiError> {
    let mut database_connection = DbConnection::from_inner(db);

    let season_option = database_connection.get_latest_active_season().await;

    match season_option {
        None => Err(ApiError::from_status(Status::NotFound)),
        Some(season) => Ok(Json(season)),
    }
}
