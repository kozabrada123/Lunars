use rocket::{get, http::Status, serde::json::Json};
use rocket_db_pools::Connection;
use rocket_okapi::openapi;

use crate::{
    database::{query::QueryParameters, DbConnection},
    response::ApiError,
    types::entities::player::Player,
    MysqlDb,
};

#[openapi(ignore = "db", tag = "Players")]
#[get("/players?<max_rating>&<min_rating>&<max_deviation>&<min_deviation>&<max_volatility>&<min_volatility>&<sort>&<limit>&<offset>")]
/// Fetches an array of all players.
///
/// Returns their current rating; does not include performance from the latest season
pub async fn get_players(
    db: Connection<MysqlDb>,
    max_rating: Option<f64>,
    min_rating: Option<f64>,
    max_deviation: Option<f64>,
    min_deviation: Option<f64>,
    max_volatility: Option<f64>,
    min_volatility: Option<f64>,
    sort: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Json<Vec<Player>> {
    let query_parameters = QueryParameters {
        max_rating,
        min_rating,
        max_deviation,
        min_deviation,
        max_volatility,
        min_volatility,
        sort,
        limit,
        offset,
        ..Default::default()
    };

    let mut database_connection = DbConnection::from_inner(db);

    Json(database_connection.get_players(query_parameters).await)
}

#[openapi(ignore = "db", tag = "Players")]
#[get("/players/search/<search_string>?<max_rating>&<min_rating>&<max_deviation>&<min_deviation>&<max_volatility>&<min_volatility>&<sort>&<limit>&<offset>")]
/// Searches for players with a similar username to the search string.
///
/// Functionally works similar to GET /players/. All query parameters from that endpoint are
/// supported and the return type is the same.
pub async fn search_players(
    db: Connection<MysqlDb>,
    search_string: String,
    max_rating: Option<f64>,
    min_rating: Option<f64>,
    max_deviation: Option<f64>,
    min_deviation: Option<f64>,
    max_volatility: Option<f64>,
    min_volatility: Option<f64>,
    sort: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
) -> Json<Vec<Player>> {
    let query_parameters = QueryParameters {
        max_rating,
        min_rating,
        max_deviation,
        min_deviation,
        max_volatility,
        min_volatility,
        sort,
        limit,
        offset,
        ..Default::default()
    };

    let mut database_connection = DbConnection::from_inner(db);

    Json(
        database_connection
            .search_players(&search_string, query_parameters)
            .await,
    )
}

#[openapi(ignore = "db", tag = "Players")]
#[get("/players/<query>")]
/// Fetches a player via an id or username.
///
/// Returns their current rating; does not include performance from the latest season.
///
/// If the query is a valid id, it will take precedence over the uesrname.
///
/// (This is why usernames shouldn't be valid ids)
///
/// If no such player is found, the [ApiError] will have code 0 and message "Not Found"
pub async fn get_player(db: Connection<MysqlDb>, query: &str) -> Result<Json<Player>, ApiError> {
    let mut database_connection = DbConnection::from_inner(db);

    let player_option = database_connection.get_player_by_id_or_name(query).await;

    match player_option {
        None => Err(ApiError::from_status(Status::NotFound)),
        Some(player) => Ok(Json(player)),
    }
}

#[openapi(ignore = "db", tag = "Players")]
#[get("/players/<query>/future")]
/// Fetches a player via an id or username.
///
/// Returns their new rating, if the season hypothetically ended right now.
///
/// (It is otherwise the same as GET /players/<query>)
pub async fn get_player_future(
    db: Connection<MysqlDb>,
    query: &str,
) -> Result<Json<Player>, ApiError> {
    let mut database_connection = DbConnection::from_inner(db);

    let player_option = database_connection.get_player_by_id_or_name(query).await;

    let mut player = match player_option {
        None => {
            return Err(ApiError::from_status(Status::NotFound));
        }
        Some(player) => player,
    };

    let active_season_res = database_connection.get_latest_season().await;
    if active_season_res.is_none() {
        return Ok(Json(player));
    }

    let active_season = active_season_res.unwrap();

    let season_completion = active_season.completion();

    let matches_for_player = database_connection
        .get_player_matches_for_season(player.id, active_season.id)
        .await;

    player.rate_player_for_elapsed_periods(matches_for_player, season_completion);

    Ok(Json(player))
}
