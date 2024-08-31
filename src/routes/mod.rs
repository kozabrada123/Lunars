//! Module containing endpoints

use serde::{Deserialize, Serialize};

pub mod players;

// Struct of the valid authentication keys
// TODO: add perms
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct AuthKey {
	 /// Sha 256 hash of the key
    hash: String,
}
