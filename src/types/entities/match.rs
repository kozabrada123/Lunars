use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{mysql::MySqlRow, FromRow, Row};


#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, PartialOrd)]
pub struct Match {
    // TODO: maybe make this uuid or at least random?
    pub id: u64,

    pub player_a: u64,
    pub player_b: u64,

    pub score_a: u8, // Score; 0 - 22
    pub score_b: u8,

    pub ping_a: u16,
    pub ping_b: u16,

    pub rating_a: f64, // Players' ranks at the time
    pub rating_b: f64,

    pub deviation_a: f64, // Players' deviations at the time
    pub deviation_b: f64,

    pub volatility_a: f64, // Players' volatilities at the time
    pub volatility_b: f64,

    pub epoch: DateTime<Utc>,
}

impl<'r> FromRow<'r, MySqlRow> for Match {
    fn from_row(row: &'r MySqlRow) -> Result<Self, sqlx::Error> {
        let id = row.try_get("id")?;

        let player_a = row.try_get("player_a")?;
        let player_b = row.try_get("player_b")?;

        let score_a = row.try_get("score_a")?;
        let score_b = row.try_get("score_b")?;

        let ping_a = row.try_get("ping_a")?;
        let ping_b = row.try_get("ping_b")?;

        let rating_a = row.try_get("rating_a")?;
        let rating_b = row.try_get("rating_b")?;

        let deviation_a = row.try_get("deviation_a")?;
        let deviation_b = row.try_get("deviation_b")?;

        let volatility_a = row.try_get("volatility_a")?;
        let volatility_b = row.try_get("volatility_b")?;

        let epoch = row.try_get("epoch")?;

        Ok(Match {
            id,
            player_a,
            player_b,
            score_a,
            score_b,
            ping_a,
            ping_b,
            rating_a,
            rating_b,
            deviation_a,
            deviation_b,
            volatility_a,
            volatility_b,
            epoch,
        })
    }
}

impl Match {
    /// Sorts a match result by a player id.
    ///
    /// makes the given player always player a
    pub fn sorted_by_player_id(mut self, player_id: u64) -> Match {
        // If the desired player is already a, we don't need to sort it at all
        if self.player_a == player_id {
            return self;
        }

        let player_a = self.player_a;
        self.player_a = self.player_b;
        self.player_b = player_a;

        let ping_a = self.ping_a;
        self.ping_a = self.ping_b;
        self.ping_b = ping_a;

        let score_a = self.score_a;
        self.score_a = self.score_b;
        self.score_b = score_a;

        let rating_a = self.rating_a;
        self.rating_a = self.rating_b;
        self.rating_b = rating_a;

        let deviation_a = self.deviation_a;
        self.deviation_a = self.deviation_b;
        self.deviation_b = deviation_a;

        let volatility_a = self.volatility_a;
        self.volatility_a = self.volatility_b;
        self.volatility_b = volatility_a;

        return self;
    }
}

/// [Match] struct with extra calculation details.
///
/// Only to be used for dummy matches and testing
#[derive(Debug, Serialize, Deserialize)]
pub struct DetailedMatch {
    // TODO: maybe make this uuid or at least random?
    pub id: u64,
    pub player_a: u64,
    pub player_b: u64,

    pub score_a: u8,
    pub score_b: u8,

    pub ping_a: u16,
    pub ping_b: u16,

    pub rating_a: u16, // Players' ratings at the time
    pub rating_b: u16,

    pub deviation_a: u16, // Players' deviations at the time
    pub deviation_b: u16,

    pub volatility_a: f64, // Players' volatilities at the time
    pub volatility_b: f64,

    pub epoch: DateTime<Utc>,

    pub debuginfo: DebugInfo,
}

impl DetailedMatch {
    pub fn new_dummy(
        player_a: u64,
        player_b: u64,
        score_a: u8,
        score_b: u8,
        ping_a: u16,
        ping_b: u16,
        rating_a: u16,
        rating_b: u16,
        deviation_a: u16,
        deviation_b: u16,
        volatility_a: f64,
        volatility_b: f64,
        debuginfo: DebugInfo,
    ) -> DetailedMatch {
        DetailedMatch {
            // Always just do 0, its not a valid match
            id: 0,
            player_a,
            player_b,

            score_a,
            score_b,

            ping_a,
            ping_b,

            rating_a,
            rating_b,

            deviation_a,
            deviation_b,

            volatility_a,
            volatility_b,

            debuginfo,

            epoch: chrono::Utc::now(),
        }
    }
}

/// Debug info for [DetailedMatch]
#[derive(Debug, Serialize, Deserialize)]
pub struct DebugInfo {
    /// time it took to process calculations, in μs
    pub time: u64,
	 /// player's ability, expressed as a u64
	 ///
	 /// Abilitys can sometimes be floats, but we can discard the .08 left over as it doesnt matter
    pub ability_a: u64,
    pub ability_b: u64,
	 /// expected score distribution, between 0 and 1
    pub expected_a: f32,
    pub expected_b: f32,
	 /// actual score distribution, between 0 and 1
    pub actual_a: f32,
    pub actual_b: f32,
}
