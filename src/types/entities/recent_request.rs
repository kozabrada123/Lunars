use std::{net::IpAddr, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlRow, FromRow, Row};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
/// A request which was processed not too long ago
pub struct RecentRequest {
    /// If we are limiting by ip, the ip that made the request
    pub ip: Option<IpAddr>,

    /// If we are limiting by api key, the sha256 hash of the ip that made the request
    pub api_key_hash: Option<String>,

    /// When the request was made, Utc time.
    pub epoch: DateTime<Utc>,
}

impl<'r> FromRow<'r, MySqlRow> for RecentRequest {
    fn from_row(row: &'r MySqlRow) -> Result<Self, sqlx::Error> {
        let ip: Option<String> = row.try_get("ip")?;

        let parsed_ip = ip.map(|x| IpAddr::from_str(x.as_ref()).unwrap());

        let api_key_hash = row.try_get("api_key_hash")?;

        let epoch = row.try_get("epoch")?;

        Ok(Self {
            ip: parsed_ip,
            api_key_hash,
            epoch,
        })
    }
}
