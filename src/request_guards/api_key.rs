use rocket::{
    async_trait,
    form::validate::Contains,
    http::Status,
    request::{FromRequest, Outcome},
};
use rocket_okapi::{
    gen::OpenApiGenerator,
    okapi::openapi3::{
        MediaType, Object, RefOr, Responses, SecurityRequirement, SecurityScheme,
        SecuritySchemeData,
    },
    request::{OpenApiFromRequest, RequestHeaderInput},
};
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

        let deserialized: Vec<ApiKey> =
            serde_json::from_str(&file_contents).expect("Failed to deserialize keyfile");

        let hashes = deserialized
            .iter()
            .map(|x| &x.hash)
            .collect::<Vec<&String>>();

        let mut hasher = sha2::Sha256::new();

        hasher.update(auth_header.as_bytes());

        let auth_header_hash = hasher.finalize();

        let hash_as_hex = hex::encode(auth_header_hash);

        if hashes.contains(&hash_as_hex) {
            return Outcome::Success(ApiKey { hash: hash_as_hex });
        }

        Outcome::Error((Status::Unauthorized, ApiError::invalid_auth()))
    }
}

// See <https://github.com/GREsau/okapi/blob/cd59fb61c5c8db1ce1976cb782efd544d8f99c38/examples/secure_request_guard/src/api_key.rs#L42C1-L135C2>
impl<'a> OpenApiFromRequest<'a> for ApiKey {
    fn from_request_input(
        _gen: &mut OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<RequestHeaderInput> {
        // Setup global requirement for Security scheme
        let security_scheme = SecurityScheme {
            description: Some("Requires an API key to access".to_owned()),
            // Setup data requirements.
            // This can be part of the `header`, `query` or `cookie`.
            data: SecuritySchemeData::ApiKey {
                name: "Authorization".to_owned(),
                location: "header".to_owned(),
            },
            extensions: Object::default(),
        };
        // Add the requirement for this route/endpoint
        // This can change between routes.
        let mut security_req = SecurityRequirement::new();
        // Each security requirement needs to be met before access is allowed.
        security_req.insert("ApiKey".to_owned(), Vec::new());
        // These vvvvvvv-----^^^^^^^^^^ values need to match exactly!
        Ok(RequestHeaderInput::Security(
            "ApiKey".to_owned(),
            security_scheme,
            security_req,
        ))
    }

    fn get_responses(gen: &mut OpenApiGenerator) -> rocket_okapi::Result<Responses> {
        let schema = gen.json_schema::<ApiError>();
        let schema_object = rocket_okapi::okapi::openapi3::Response {
            description: "\
        # 401 Unauthorized\n\
        No authentication was given or it was invalid. \
        "
            .to_owned(),
            content: rocket_okapi::okapi::map! {
                "application/json".to_owned() => MediaType {
                    schema: Some(schema),
                    ..Default::default()
                }
            },
            ..Default::default()
        };

        Ok(Responses {
            // Recommended and most strait forward.
            // And easy to add or remove new responses.
            responses: rocket_okapi::okapi::map! {
                "401".to_owned() => RefOr::Object(schema_object),
            },
            ..Default::default()
        })
    }
}
