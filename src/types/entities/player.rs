use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlRow, FromRow, Row};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd)]
pub struct Player {
    // TODO: maybe make this uuid or at least random?
    pub id: u64,
    pub name: String,
    pub rating: f64,
    pub deviation: f64,
    pub volatility: f64,
}

impl<'r> FromRow<'r, MySqlRow> for Player {
    fn from_row(row: &'r MySqlRow) -> Result<Self, sqlx::Error> {
        let id = row.try_get("id")?;
        let name = row.try_get("name")?;
        let rating = row.try_get("rating")?;
        let deviation = row.try_get("deviation")?;
        let volatility = row.try_get("volatility")?;

        Ok(Player {
            id,
            name,
            rating,
            deviation,
            volatility,
        })
    }
}
