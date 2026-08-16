use rocket::{
    http::Status,
    request::{FromRequest, Outcome},
    Request,
};
use rocket_okapi::{okapi::openapi3::Responses, request::OpenApiFromRequest};
use std::net::IpAddr;

#[derive(Debug, Clone, Copy)]
pub enum RealIpAddrError {
    /// We have reverse proxing enabled, but no x headers were provided; your configuration is funked
    XHeaderMissing,
    /// X-Real-Ip was present, but we failed to parse it
    FailedToParse,
    /// The request had X-Real-IP, but no reverse proxy is enabled or the request didn't provide it - attempted attack?
    XHeaderUnexpected,
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
        let use_reverse_proxy_str = std::env::var("USE_REVERSE_PROXY").ok().unwrap_or_default().to_lowercase();

        let use_reverse_proxy = use_reverse_proxy_str.contains("true") || use_reverse_proxy_str.contains("yes") || use_reverse_proxy_str.contains("1");

        if use_reverse_proxy {
            let real_ip = request.headers().get_one("X-Real-IP");

            if real_ip.is_none() {
                log::error!(
                    "Failed to get X-Real-IP header, even though we should be behind a reverse proxy! Please check your configuration"
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
            log::error!("X-Real-IP header was present, but we shouldn't be behind a reverse proxy!");
            log::warn!("If you are running in a reverse proxy, please set USE_REVERSE_PROXY=\"true\" in your .env");
            log::warn!("If you are NOT running in a reverse proxy, someone might have just tried to attack you.");
            log::warn!(
                "'Real' ip: {:?}, remote: {:?}",
                fake_real_ip,
                request.remote()
            );
            return Outcome::Error((
                Status::InternalServerError,
                RealIpAddrError::XHeaderUnexpected,
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
