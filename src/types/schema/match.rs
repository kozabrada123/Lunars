use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// Struct of a match to add
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, JsonSchema)]
pub struct AddMatchSchema {
    /// Username of the first player
    pub player_a: String,
    /// Username of the second player
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
