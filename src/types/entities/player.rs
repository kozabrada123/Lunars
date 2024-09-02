use rocket_okapi::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlRow, FromRow, Row};

use crate::glicko::{default_deviation, default_rating, default_volatility};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, PartialOrd, JsonSchema)]
#[schemars(example = "player_example_default_glicko")]
pub struct Player {
    #[schemars(example = "example_id")]
    // TODO: maybe make this uuid or at least random?
    pub id: u64,
    #[schemars(example = "example_username")]
    pub name: String,
    #[schemars(example = "default_rating")]
    /// The center point of the player's rating range.
    pub rating: f64,
    #[schemars(example = "default_deviation")]
    /// The deviation of the rating.
    ///
    /// One point is equal to one standard deviation (?)
    ///
    /// To show a 95% accurate range of the player's rating,
    /// calculate rating ± 1.96 * deviation
    pub deviation: f64,
    #[schemars(example = "default_volatility")]
    /// A measure of how (in)consistent the player is
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

pub fn example_username() -> String {
    "toucan175".to_string()
}

pub fn example_id() -> u64 {
    53
}

fn player_example_default_glicko() -> Player {
    Player {
        id: example_id(),
        name: example_username(),
        rating: default_rating(),
        deviation: default_deviation(),
        volatility: default_volatility(),
    }
}
