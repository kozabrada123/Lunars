use serde::{Deserialize, Serialize};

// Struct of player we add
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, PartialOrd)]
struct AddPlayerSchema {
    pub name: String,
    pub rating: f64,
    pub deviation: f64,
    pub volatility: f64,
}
