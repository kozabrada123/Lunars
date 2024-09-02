use crate::response::ApiError;
use rocket::get;
use rocket_okapi::openapi;

pub mod fairing;

pub const UNAUTHENTICATED_ALLOWED_REQUESTS_PER_MINUTE: usize = 60;
pub const API_KEY_ALLOWED_REQUESTS_PER_MINUTE: usize = 120;

#[openapi(skip)]
#[get("/.super_secret/you_are_rate_limited")]
/// Gets the "you are ratelimited!" error
pub async fn get_ratelimited_error() -> ApiError {
    ApiError::ratelimited()
}
