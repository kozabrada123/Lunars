//! Module containing endpoints

use serde::{Deserialize, Serialize};

pub mod catchers;
pub mod matches;
pub mod players;
pub mod system;

// Struct of the valid authentication keys
// TODO: add perms
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct AuthKey {
    /// Sha 256 hash of the key
    hash: String,
}
