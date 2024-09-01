use rocket::{get, http::Status, serde::json::Json};
use rocket_db_pools::Connection;
use rocket_okapi::openapi;

use crate::{
    database::{query::QueryParameters, DbConnection},
    request_guards::chrono::chrono_timestamp_from_string,
    response::ApiError,
    types::entities::r#match::Match,
    MysqlDb,
};

#[openapi(ignore = "db", tag = "Matches")]
#[get("/matches?<after>&<before>&<has_player>&<sort>&<limit>&<offset>")]
/// Fetches an array of all players.
///
/// Here ?after and ?before can be used to target when the matches were submittewere submitted (in Utc time)
///
/// They can be set to either an rfc3339 (iso) timestamp or unix milliseconds
pub async fn get_matches(
    db: Connection<MysqlDb>,
    after: Option<String>,
    before: Option<String>,
    has_player: Option<Vec<String>>,
    sort: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Json<Vec<Match>> {
    let mut after_chrono = None;
    let mut before_chrono = None;

    if let Some(after_timestamp) = after {
        after_chrono = chrono_timestamp_from_string(&after_timestamp);
    }

    if let Some(before_timestamp) = before {
        before_chrono = chrono_timestamp_from_string(&before_timestamp);
    }

    let query_parameters = QueryParameters {
        after: after_chrono,
        before: before_chrono,
        has_player,
        sort,
        limit,
        offset,
        ..Default::default()
    };

    let mut database_connection = DbConnection::from_inner(db);

    Json(database_connection.get_matches(query_parameters).await)
}

#[openapi(ignore = "db", tag = "Matches")]
#[get("/matches/<id>")]
/// Fetches a match via its id.
///
/// If no such match is found, the [ApiError] will have code 0 and message "Not Found"
pub async fn get_match(db: Connection<MysqlDb>, id: u64) -> Result<Json<Match>, ApiError> {
    let mut database_connection = DbConnection::from_inner(db);

    let match_option = database_connection.get_match_by_id(id).await;

    match match_option {
        None => Err(ApiError::from_status(Status::NotFound)),
        Some(a_match) => Ok(Json(a_match)),
    }
}
