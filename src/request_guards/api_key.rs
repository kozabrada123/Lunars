use rocket::{async_trait, form::validate::Contains, http::Status, request::{FromRequest, Outcome}};
use serde::{Deserialize, Serialize};
use sha2::Digest;

use crate::response::ApiError;

/// Struct representing a requester with a valid api key.
// TODO: add perms
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone, PartialOrd, Ord, Hash)]
pub struct ApiKey {
    pub hash: String,
}

#[async_trait]
impl<'r> FromRequest<'r> for ApiKey {
	type Error = ApiError;

	async fn from_request(request: &'r rocket::Request<'_>) -> Outcome<Self, Self::Error> {
        let auth_header_option = request.headers().get_one("Authorization");

		  if auth_header_option.is_none() {
				return Outcome::Error((Status::Unauthorized, ApiError::invalid_auth()));
		  }

		  let auth_header = auth_header_option.unwrap();

		  let auth_file = std::env::var("KEYFILE").expect("You have not specified a keyfile!");

		  let file_contents = std::fs::read_to_string(auth_file).expect("Failed to read keyfile");

		  let deserialized: Vec<ApiKey> = serde_json::from_str(&file_contents).expect("Failed to deserialize keyfile");

		  let hashes = deserialized.iter().map(|x| &x.hash).collect::<Vec<&String>>();

		  let mut hasher = sha2::Sha256::new();

		  hasher.update(auth_header.as_bytes());

		  let auth_header_hash = hasher.finalize();

		  let hash_as_hex = hex::encode(auth_header_hash);

		  if hashes.contains(&hash_as_hex) {
				return Outcome::Success(ApiKey {hash: hash_as_hex});
		  }

		  Outcome::Error((Status::Unauthorized, ApiError::invalid_auth()))
	}
}
