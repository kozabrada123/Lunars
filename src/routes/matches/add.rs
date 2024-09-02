use chrono::Utc;
use log::info;
use rocket::{http::Status, post, serde::json::Json};
use rocket_db_pools::Connection;
use rocket_okapi::openapi;

use crate::{
    database::DbConnection,
    request_guards::api_key::ApiKey,
    response::ApiError,
    types::{
        entities::r#match::Match,
        schema::r#match::{AddMatchReturnSchema, AddMatchSchema},
    },
    MysqlDb,
};

#[openapi(ignore = "db", tag = "Matches")]
#[post("/api/matches", data = "<schema>")]
#[allow(unused)]
/// Adds a match to the latest rating period.
///
/// Requires authorization.
///
/// Has a special return type which includes the created match
/// along with the new live ratings of the two players.
///
/// Returns a 404 if either one of the two players don't exist.
///
/// Returns an error with code 5 if player_a is player_b, since players usually
/// do not play against themselves.
pub async fn add_match(
    db: Connection<MysqlDb>,
    api_key: ApiKey,
    schema: Json<AddMatchSchema>,
) -> Result<Json<AddMatchReturnSchema>, ApiError> {
    let mut database_connection = DbConnection::from_inner(db);

    let started = std::time::Instant::now();

    let player_a_res = database_connection
        .get_player_by_id_or_name(&schema.player_a)
        .await;
    if player_a_res.is_none() {
        return Err(ApiError::from_status(Status::NotFound));
    }
    let mut player_a = player_a_res.unwrap();

    let player_b_res = database_connection
        .get_player_by_id_or_name(&schema.player_b)
        .await;
    if player_b_res.is_none() {
        return Err(ApiError::from_status(Status::NotFound));
    }
    let mut player_b = player_b_res.unwrap();

    if player_a.id == player_b.id {
        log::warn!(
            "Tried to submit a match between where {} played against themselves",
            player_a.name
        );
        return Err(ApiError::match_player_a_is_player_b());
    }

    let current_rating_period = database_connection
        .get_latest_active_season()
        .await
        .unwrap();

    let now = Utc::now();

    let mut a_match = Match {
        id: 0,
        rating_period: current_rating_period.id,
        player_a: player_a.id,
        player_b: player_b.id,
        rating_a: player_a.rating,
        rating_b: player_b.rating,
        deviation_a: player_a.deviation,
        deviation_b: player_b.deviation,
        volatility_a: player_a.volatility,
        volatility_b: player_b.volatility,
        ping_a: schema.ping_a,
        ping_b: schema.ping_b,
        score_a: schema.score_a,
        score_b: schema.score_b,
        epoch: now,
    };

    let result = database_connection.add_match(&a_match).await.unwrap();

    a_match.id = result.last_insert_id();

    // Compute live ratings
    let season_completion = current_rating_period.completion();

    let player_a_matches = database_connection
        .get_player_matches_for_season(player_a.id, current_rating_period.id)
        .await;
    let player_b_matches = database_connection
        .get_player_matches_for_season(player_b.id, current_rating_period.id)
        .await;

    let math_started = std::time::Instant::now();

    player_a.rate_player_for_elapsed_periods(player_a_matches, season_completion);
    player_b.rate_player_for_elapsed_periods(player_b_matches, season_completion);

    let math_elapsed = math_started.elapsed();
    let elapsed = started.elapsed();

    info!(
        "POST /matches/ took {:?}, {:?} of that was math",
        elapsed, math_elapsed
    );

    let return_schema = AddMatchReturnSchema {
        live_a: player_a,
        live_b: player_b,
        created: a_match,
    };

    Ok(Json(return_schema))
}

#[openapi(ignore = "db", tag = "Matches")]
#[post("/api/matches/dummy", data = "<schema>")]
#[allow(unused)]
/// Runs the calculations after a match, but does not actually change any data.
///
/// Has a special return type which includes how the created match would look
/// along with the hypothetical new live ratings of the two players.
///
/// Returns a 404 if either one of the two players don't exist.
///
/// Returns an error with code 5 if player_a is player_b, since players usually
/// do not play against themselves.
///
/// (Behaves similarly to POST /matches/)
pub async fn add_match_dummy(
    db: Connection<MysqlDb>,
    schema: Json<AddMatchSchema>,
) -> Result<Json<AddMatchReturnSchema>, ApiError> {
    let mut database_connection = DbConnection::from_inner(db);

    let started = std::time::Instant::now();

    let player_a_res = database_connection
        .get_player_by_id_or_name(&schema.player_a)
        .await;
    if player_a_res.is_none() {
        return Err(ApiError::from_status(Status::NotFound));
    }
    let mut player_a = player_a_res.unwrap();

    let player_b_res = database_connection
        .get_player_by_id_or_name(&schema.player_b)
        .await;
    if player_b_res.is_none() {
        return Err(ApiError::from_status(Status::NotFound));
    }
    let mut player_b = player_b_res.unwrap();

    if player_a.id == player_b.id {
        log::warn!(
            "Tried to submit a match between where {} played against themselves",
            player_a.name
        );
        return Err(ApiError::match_player_a_is_player_b());
    }

    let current_rating_period = database_connection
        .get_latest_active_season()
        .await
        .unwrap();

    let now = Utc::now();

    let mut a_match = Match {
        id: 0,
        rating_period: current_rating_period.id,
        player_a: player_a.id,
        player_b: player_b.id,
        rating_a: player_a.rating,
        rating_b: player_b.rating,
        deviation_a: player_a.deviation,
        deviation_b: player_b.deviation,
        volatility_a: player_a.volatility,
        volatility_b: player_b.volatility,
        ping_a: schema.ping_a,
        ping_b: schema.ping_b,
        score_a: schema.score_a,
        score_b: schema.score_b,
        epoch: now,
    };

    // Compute live ratings
    let season_completion = current_rating_period.completion();

    let mut player_a_matches = database_connection
        .get_player_matches_for_season(player_a.id, current_rating_period.id)
        .await;
    let mut player_b_matches = database_connection
        .get_player_matches_for_season(player_b.id, current_rating_period.id)
        .await;

    player_a_matches.push(a_match.clone());
    player_b_matches.push(a_match.clone());

    let math_started = std::time::Instant::now();

    player_a.rate_player_for_elapsed_periods(player_a_matches, season_completion);
    player_b.rate_player_for_elapsed_periods(player_b_matches, season_completion);

    let math_elapsed = math_started.elapsed();
    let elapsed = started.elapsed();

    info!(
        "POST /matches/dummy took {:?}, {:?} of that was math",
        elapsed, math_elapsed
    );

    let return_schema = AddMatchReturnSchema {
        live_a: player_a,
        live_b: player_b,
        created: a_match,
    };

    Ok(Json(return_schema))
}
