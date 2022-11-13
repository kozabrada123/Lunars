// Internal glicko functions
// Holy shit thanks to https://github.com/deepy/glicko2/blob/master/glicko2/glicko2.py
// I literally stole all their code
// I still lost 87 years of my lifespan

// This code very closely resembles https://github.com/deepy/glicko2/blob/master/glicko2/glicko2.py, I tried to it myself but it was too painful
// So I copied that

use crate::db;

// System's tau, constains the volatility change over time.
const TAU: f64 = 0.5;

// How much we inflate the value for our users.
// The python thing had 173.7178
const CONVERSION_VAL: f64 = 173.7178;

// Please dont touch these!!
// These are the set values a player gets when joining the system.
const DEFAULT_RATING: u16 = 1500;
const DEFAULT_DEVIATION: u16 = 350;
const DEFAULT_VOLATILITY: f64 = 0.06;

// Function that gets a player's rating for showing
// Basically normalizes
fn get_rating(player: db::Player) -> u16 {
    return ((player.rank as f64 * CONVERSION_VAL) + DEFAULT_RATING as f64) as u16;
}

// Sets a rating to a player
// Uses normalized values
fn set_rating(player: &mut db::Player, new_rank: u16) {
    player.rank = ((new_rank - DEFAULT_RATING) as f64 / CONVERSION_VAL) as u16;
}

// Same as get rating, normalizes the deviation
fn get_Rd(player: db::Player) -> u16 {
    return (player.deviation as f64 * CONVERSION_VAL) as u16;
}

// Same as set rating, sets the value from the normalized
fn set_Rd(player: &mut db::Player, new_rd: u16) {
    player.deviation = (new_rd as f64 / CONVERSION_VAL) as u16;
}

// Resets / sets a player to default stats
