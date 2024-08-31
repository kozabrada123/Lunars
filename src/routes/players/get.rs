use rocket::{get, serde::json::Json};
use rocket_db_pools::Connection;

use crate::{database::DbConnection, response::ApiError, types::entities::player::Player, MysqlDb};
use crate::request_guards::api_key::ApiKey;


#[get("/")]
pub async fn get_players(db: Connection<MysqlDb>) -> Result<Json<Vec<Player>>, ApiError> {
	let mut database_connection = DbConnection::from_inner(db);

	Ok(Json(database_connection.get_players().await))
}
