use serde::{Deserialize, Serialize};

// Struct of a match to add
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct AddMatchSchema {
    player_a: String,
    player_b: String,
    ping_a: u16,
    ping_b: u16,
    score_a: u8,
    score_b: u8,
}

