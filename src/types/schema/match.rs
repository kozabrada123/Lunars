use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::types::entities::{player::Player, r#match::Match};

// Struct of a match to add
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct AddMatchSchema {
    /// Username or id of the first player;
    ///
    /// id takes priority over username, like the GET /players/<query> endpoint
    pub player_a: String,
    /// Username or id of the second player
    ///
    /// id takes priority over username, like the GET /players/<query> endpoint
    pub player_b: String,
    /// Ping of the first player. 0 - 65000
    pub ping_a: u16,
    /// Ping of the second player. 0 - 65000
    pub ping_b: u16,
    /// Score of the first player. 0 - 22
    pub score_a: u8,
    /// Score of the second player. 0 - 22
    pub score_b: u8,
}

// Return type of the add match endpoint.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd, JsonSchema)]
pub struct AddMatchReturnSchema {
    /// The created match
    pub created: Match,
    /// Player_a's new live rating
    pub live_a: Player,
    /// Player_b's new live rating
    pub live_b: Player,
}
