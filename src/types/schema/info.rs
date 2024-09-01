use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::glicko::{default_deviation, default_rating, default_volatility, ping_influence, rating_period_duration_days, tau};

#[derive(Clone, Serialize, Deserialize, PartialEq, PartialOrd, Debug, JsonSchema)]
#[schemars(example = "InstanceConstants::default")]
/// Schema for information about the system / instance's constants
pub struct InstanceConstants {
    #[schemars(example = "default_rating")]
    pub default_rating: f64,
    #[schemars(example = "default_deviation")]
    pub default_deviation: f64,
    #[schemars(example = "default_volatility")]
    pub default_volatility: f64,
    #[schemars(example = "tau")]
    pub tau: f64,
    #[schemars(example = "ping_influence")]
    pub ping_influence: f64,
	 #[schemars(example = "rating_period_duration_days")]
    pub rating_period_duration_days: u64,
}

impl Default for InstanceConstants {
    fn default() -> Self {
        InstanceConstants {
            default_rating: default_rating(),
            default_deviation: default_deviation(),
            default_volatility: default_volatility(),
            ping_influence: ping_influence(),
            tau: tau(),
				rating_period_duration_days: rating_period_duration_days(),
        }
    }
}
