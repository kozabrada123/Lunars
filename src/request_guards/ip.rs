use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    Request,
};
use rocket_okapi::{okapi::openapi3::Responses, request::OpenApiFromRequest};
use sha2::Digest;
use std::net::IpAddr;

#[derive(Debug, Clone, Copy)]
pub enum RealIpAddrError {
    /// We have reverse proxy key set, but no x headers were provided; your configuration is funked
    XHeaderMissing,
    /// X-Real-Ip was present, but we failed to parse it
    FailedToParse,
    /// The request had X-Real-IP, but no reverse proxy key is set or the request didn't provide it - attempted attack?
    ReverseProxyKeyMissing,
    /// The request has X-Real-IP and reverse proxy key, but the reverse proxy key was invalid
    ReverseProxyKeyInvalid,
    /// Failed to get rocket remote
    RemoteFailed,
}

#[derive(Debug, Copy, Clone)]
/// Provides a request guard for a client's real ip, based off the X-Real-IP header
pub struct RealIpAddr {
    pub ip_addr: IpAddr,
}

impl From<RealIpAddr> for IpAddr {
    fn from(value: RealIpAddr) -> Self {
        value.ip_addr
    }
}

impl std::fmt::Display for RealIpAddr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.ip_addr.fmt(f)
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for RealIpAddr {
    type Error = RealIpAddrError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let reverse_proxy_key_hash_res = std::env::var("REVERSE_PROXY_KEY_HASH").ok();

        if let Some(proxy_key_hash) = reverse_proxy_key_hash_res {
            // Does the request have the right proxy key hash?
            let provided_proxy_key_res = request.headers().get_one("X-Reverse-Proxy-Key");

            if provided_proxy_key_res.is_none() {
                log::warn!("A reverse proxy key is configured, but was not provided. Is your configuration correct?");
                log::warn!("Remote: {:?}", request.remote());

                return Outcome::Error((
                    Status::InternalServerError,
                    RealIpAddrError::ReverseProxyKeyMissing,
                ));
            }

            let provided_proxy_key = provided_proxy_key_res.unwrap();

            let mut hasher = sha2::Sha256::new();

            hasher.update(provided_proxy_key.as_bytes());

            let provided_proxy_key_hash_bytes = hasher.finalize();

            let provided_proxy_key_hash = hex::encode(provided_proxy_key_hash_bytes);

            if provided_proxy_key_hash != proxy_key_hash {
                log::warn!("Request has invalid proxy key hash");
                log::warn!("Remote: {:?}", request.remote());

                return Outcome::Error((
                    Status::InternalServerError,
                    RealIpAddrError::ReverseProxyKeyInvalid,
                ));
            }

            // Did the request provide a real ip?
            let real_ip = request.headers().get_one("X-Real-IP");

            if real_ip.is_none() {
                log::error!(
                    "Failed to get X-Real-IP header, even though reverse proxy key is set!"
                );

                return Outcome::Error((
                    Status::InternalServerError,
                    RealIpAddrError::XHeaderMissing,
                ));
            }

            let real_ip_string = real_ip.unwrap();

            let parsed_ip = real_ip_string.parse::<IpAddr>();

            if parsed_ip.is_err() {
                log::error!("Failed to parse real ip: {:?}", real_ip_string,);

                return Outcome::Error((
                    Status::InternalServerError,
                    RealIpAddrError::FailedToParse,
                ));
            }

            return Outcome::Success(Self {
                ip_addr: parsed_ip.unwrap(),
            });
        }

        // Are we getting bamboozled?
        let x_real_ip_res = request.headers().get_one("X-Real-IP");

        if let Some(fake_real_ip) = x_real_ip_res {
            log::error!("X-Real-IP header was present, but no reverse proxy key is set!");
            log::error!("If you are running in a reverse proxy, you have forgotten to set the key");
            log::error!("If you are NOT running in a reverse proxy, someone might have just tried to attack you.");
            log::error!(
                "'Real' ip: {:?}, remote: {:?}",
                fake_real_ip,
                request.remote()
            );
            return Outcome::Error((
                Status::InternalServerError,
                RealIpAddrError::ReverseProxyKeyMissing,
            ));
        }

        let bare_remote = request.remote();

        if let Some(socket_addr) = bare_remote {
            return Outcome::Success(Self {
                ip_addr: socket_addr.ip(),
            });
        }

        log::error!("Failed to get request remote");
        return Outcome::Error((Status::InternalServerError, RealIpAddrError::RemoteFailed));
    }
}

// This should not be documented
impl<'r> OpenApiFromRequest<'r> for RealIpAddr {
    fn get_responses(
        _gen: &mut rocket_okapi::gen::OpenApiGenerator,
    ) -> rocket_okapi::Result<rocket_okapi::okapi::openapi3::Responses> {
        Ok(Responses::default())
    }

    fn from_request_input(
        _gen: &mut rocket_okapi::gen::OpenApiGenerator,
        _name: String,
        _required: bool,
    ) -> rocket_okapi::Result<rocket_okapi::request::RequestHeaderInput> {
        Ok(rocket_okapi::request::RequestHeaderInput::None)
    }
}
