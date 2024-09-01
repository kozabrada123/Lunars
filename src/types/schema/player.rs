use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::entities::player::example_username;

// Struct of a player we add
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, JsonSchema)]
pub struct AddPlayerSchema {
    #[schemars(example = "example_username")]
    /// Warframe username of the player we're adding.
    ///
    /// (must be unique, shouldn't be a valid integer)
    pub name: String,
    /// Optionally you can provide the rating of the player.
    ///
    /// If none is provided, the default of the system will be used.
    pub rating: Option<f64>,
    /// Optionally you can provide the rating deviation of the player.
    ///
    /// If none is provided, the default of the system will be used.
    pub deviation: Option<f64>,
    /// Optionally you can provide the rating volatility of the player.
    ///
    /// If none is provided, the default of the system will be used.
    pub volatility: Option<f64>,
}
