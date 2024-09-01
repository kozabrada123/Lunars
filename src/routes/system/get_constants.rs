use rocket::get;
use rocket::serde::json::Json;
use rocket_okapi::openapi;

use crate::types::schema::info::InstanceConstants;

#[openapi(tag = "System")]
#[get("/system/constants")]
/// Returns the system constants used for the ranking system.
pub async fn get_system_constants() -> Json<InstanceConstants> {
    Json(InstanceConstants::default())
}
