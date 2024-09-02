use regex::Regex;
use rocket::{post, serde::json::Json};
use rocket_db_pools::Connection;
use rocket_okapi::openapi;

use crate::{
    database::DbConnection,
    glicko::{default_deviation, default_rating, default_volatility},
    request_guards::api_key::ApiKey,
    response::ApiError,
    types::{entities::player::Player, schema::player::AddPlayerSchema},
    MysqlDb,
};

#[openapi(ignore = "db", tag = "Players")]
#[post("/api/players", data = "<schema>")]
#[allow(unused)]
/// Adds a player to the database.
///
/// Requires authorization.
///
/// Returns the player object if it was successfully added.
///
/// Returns an error with code 3 if the username is already taken.
///
/// Returns an error with code 4 if the username is invalid.
///
/// This can happen for one of two reasons:
/// - the username does not match warframe's username system (regex: [A-Za-z0-9_.-]{2,24}(#\d{3})? )
/// - the username is a valid u64 id. These are not allowed since some endpoints accept either an
/// id or username
pub async fn add_player(
    db: Connection<MysqlDb>,
    api_key: ApiKey,
    schema: Json<AddPlayerSchema>,
) -> Result<Json<Player>, ApiError> {
    let mut database_connection = DbConnection::from_inner(db);

    let existing_player_option = database_connection.get_player_by_name(&schema.name).await;

    match existing_player_option {
        None => {}
        Some(_player) => {
            return Err(ApiError::username_already_taken());
        }
    }

    if schema.name.parse::<u64>().is_ok() {
        log::warn!("Tried to add player with numeric username: {}", schema.name);
        return Err(ApiError::invalid_username("Username cannot be a valid id."));
    }

    // Validate username
    if !Regex::new(r#"[A-Za-z0-9_.-]{2,24}(#\d{3})?"#)
        .unwrap()
        .is_match(&schema.name)
    {
        log::warn!(
            "Tried to add player with username that doesn't match regex: {}",
            schema.name
        );
        return Err(ApiError::invalid_username(
            "Username is not a valid warframe username.",
        ));
    }

    let mut player = Player {
        id: 0,
        name: schema.name.clone(),
        rating: schema.rating.unwrap_or(default_rating()),
        deviation: schema.deviation.unwrap_or(default_deviation()),
        volatility: schema.deviation.unwrap_or(default_volatility()),
    };

    let result = database_connection.add_player(&player).await.unwrap();

    // Return the id of the player we added
    player.id = result.last_insert_id();

    Ok(Json(player))
}
