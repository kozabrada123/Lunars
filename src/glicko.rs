//! Internal glicko logic

// Holy shit thanks to https://github.com/deepy/glicko2/blob/master/glicko2/glicko2.py
// This code very closely resembles https://github.com/deepy/glicko2/blob/master/glicko2/glicko2.py
// Thanks so much to them, they made this implementation wayyy easier

use std::f64::consts::PI;

use rocket::form::validate::Len;

use crate::types::entities::{player::*, r#match::*};

/// System's tau, constains the volatility change over time.
const TAU: f64 = 0.5;

/// How much we inflate the value for our users.
// The python thing had 173.7178
const RATING_CONVERSION_CONSTANT: f64 = 173.7178;

// Please dont touch these!!
// These are the set values a player gets when joining the system.
const DEFAULT_RATING: u16 = 1500;
const DEFAULT_DEVIATION: u16 = 350;
const DEFAULT_VOLATILITY: f64 = 0.06;

/// Function that normalizes a player's rating for showing
pub fn rating_to_public(rating: f64) -> u16 {
    return ((rating as f64 * RATING_CONVERSION_CONSTANT) + DEFAULT_RATING as f64) as u16;
}

/// Function that normalizes a player's deviation for showing
pub fn deviation_to_public(deviation: f64) -> u16 {
    return (deviation as f64 * RATING_CONVERSION_CONSTANT) as u16;
}

/// Function that un-normalizes a player's rating
pub fn rating_from_public(public_rating: u16) -> f64 {
    (public_rating as f64 - DEFAULT_RATING as f64) as f64 / RATING_CONVERSION_CONSTANT
}

/// Function that un-normalizes a player's deviation
pub fn deviation_from_public(deviation: u16) -> f64 {
    deviation as f64 / RATING_CONVERSION_CONSTANT
}

impl Player {
    /// Function that gets a player's rating for showing
    pub fn get_public_rating(&self) -> u16 {
        rating_to_public(self.rating)
    }

    /// Sets a rating for a player
    ///
    /// uses normalized / public values
    pub fn set_public_rating(&mut self, new_rating: u16) {
        self.rating = rating_from_public(new_rating);
    }

    /// Same as [Self::get_public_rating], normalizes the deviation
    pub fn get_public_deviation(&self) -> u16 {
        deviation_to_public(self.deviation)
    }

    /// Same as set rating, sets the value from the normalized
    pub fn set_public_deviation(&mut self, new_deviation: u16) {
        self.deviation = deviation_from_public(new_deviation);
    }

    /// Resets / sets a player to default stats
    pub fn reset_defaults(&mut self) {
        self.set_public_rating(DEFAULT_RATING);
        self.set_public_deviation(DEFAULT_DEVIATION);
        self.volatility = DEFAULT_VOLATILITY;
    }

    /// Applies step 6, used for players who didn't compete.
    fn update_if_did_not_compete(&mut self) {
        self._set_pre_rating_deviation();
    }

    /// "Calculates and updates the player's rating deviation for the
    /// beginning of a rating period. "
    fn _set_pre_rating_deviation(&mut self) {
        self.deviation = (self.deviation.powi(2) + self.volatility.powi(2)).sqrt();
    }

    /// Calculates and updates the new rating and deviation for a player.
    pub fn update_player(&mut self, matches: Vec<Match>) {
        // Convert the values for internal use

        // First also sort the matches so player_a is always us
        let mut matches_converted = Vec::new();

        for game_match in matches {
            matches_converted.push(game_match.sorted_by_player_id(self.id));
        }

        // Now convert the values from readable to internal
        for game_match in matches_converted.iter_mut() {
            // Only do the b ones since A is us
            game_match.rating_b = rating_from_public(game_match.rating_b as u16);
            game_match.deviation_b = deviation_from_public(game_match.deviation_b as u16);
        }

        // Calculate anchillary v
        let v = calculate_v(&self, &matches_converted);

        // Calculate volatility
        self.volatility = calculate_volatility(&self, &matches_converted, v);

        self._set_pre_rating_deviation();

        // Calculate our deviation
        self.deviation = 1.0 / ((1.0 / self.deviation.powi(2)) + (1.0 / v)).sqrt();

        // Calculate our rating
        let mut temp_sum = 0.0;
        for game_match in matches_converted {
            temp_sum += calculate_g(game_match.deviation_b)
                * (calculate_match_a_score(&game_match)
                    - calculate_e(&self, game_match.rating_b, game_match.deviation_b));
        }

        self.rating = self.deviation.powi(2) * temp_sum;
    }
}

// Calculates the new volatility from matches
fn calculate_volatility(player: &Player, matches: &Vec<Match>, v: f64) -> f64 {
    // Step 1:
    let a = player.volatility.powi(2).ln();
    let eps = 0.000001;
    let mut big_a = a;

    // Step 2:
    let mut big_b: f64;
    let delta = calculate_delta(&player, matches, v);
    let tau = TAU;

    if delta.powi(2) > (player.deviation.powi(2) + v) {
        big_b = (delta.powi(2) - player.deviation.powi(2) - v).ln();
    } else {
        let mut k = 1;
        while calculate_f(&player, a - k as f64 * tau.powi(2).sqrt(), delta, v, a) < 0.0 {
            k += 1;
        }
        big_b = a - k as f64 * tau.powi(2).sqrt();
    }

    // Step 3:
    let mut f_a = calculate_f(&player, big_a, delta, v, a);
    let mut f_b = calculate_f(&player, big_b, delta, v, a);

    // Step 4:
    while (big_b - big_a).abs() > eps {
        // A
        let big_c = big_a + ((big_a - big_b) * f_a) / (f_b - f_a);
        let f_c = calculate_f(&player, big_c, delta, v, a);

        // B
        if f_c * f_b <= 0.0 {
            big_a = big_b;
            f_a = f_b;
        } else {
            f_a = f_a / 2.0;
        }

        // C
        big_b = big_c;
        f_b = f_c;
    }

    // Step 5:
    (big_a / 2.0).exp()
}

// F func from glicko
fn calculate_f(player: &Player, x: f64, delta: f64, v: f64, a: f64) -> f64 {
    let ex = x.exp();

    let num1 = ex * (delta.powi(2) - player.rating.powi(2) - v - ex);

    let denom1 = 2.0 * ((player.rating.powi(2) + v + ex).powi(2));

    (num1 / denom1) - ((x - a) / (TAU.powi(2)))
}

// The Delta func from glicko.
fn calculate_delta(player: &Player, matches: &Vec<Match>, v: f64) -> f64 {
    let mut temp_sum = 0.0;
    for game_match in matches {
        temp_sum += calculate_g(game_match.deviation_b)
            * (calculate_match_a_score(game_match) // Only difference here is our outcome is 0 - 1 when in glicko its 0 || 1
                - calculate_e(player, game_match.rating_b, game_match.deviation_b));
    }

    return v * temp_sum;
}

// v func from glicko
fn calculate_v(player: &Player, matches: &Vec<Match>) -> f64 {
    let mut temp_sum: f64 = 0.0;

    for game_match in matches {
        let temp_e = calculate_e(&player, game_match.rating_b, game_match.deviation_b);
        temp_sum += calculate_g(game_match.deviation_b).powi(2) * temp_e * (1.0 - temp_e);
    }

    1.0 / temp_sum
}

// e func from glicko
fn calculate_e(player: &Player, rating_b: f64, deviation_b: f64) -> f64 {
    1.0 / (1.0
        + (-1.0 * calculate_g(deviation_b) * (player.rating - rating_b)).exp())
}

// g func from glicko
fn calculate_g(deviation: f64) -> f64 {
    1.0 / (1.0 + 3.0 * deviation.powi(2) / PI.powi(2)).sqrt()
}

// Only function not stolen and not in glicko, processes a match to a 0 - 1 float of how well player a did
fn calculate_match_a_score(game_match: &Match) -> f64 {
    game_match.score_a as f64 / (game_match.score_a + game_match.score_b) as f64
}

/// See <http://www.glicko.net/glicko/glicko2.pdf> (Example calculation)
#[test]
fn math_is_mathing() {
    let mut test_1 = Player {
        id: 1,
        name: "Test1".to_string(),
        deviation: 0.0,
        rating: 0.0,
        volatility: 0.06,
    };

	 test_1.set_public_rating(1500);
	 test_1.set_public_deviation(200);

    println!("Starting r': {}", test_1.get_public_rating());
    println!("Starting RD': {}", test_1.get_public_deviation());

    let vec_matches = vec![Match {
        player_a: 1,
        player_b: 2,
        id: 0,
        ping_a: 0,
        ping_b: 0,
        rating_a: test_1.rating,
        rating_b: 1400.0,
		  deviation_a: test_1.deviation,
        deviation_b: 30.0,
	     volatility_a: test_1.volatility,
        volatility_b: DEFAULT_VOLATILITY,
        score_a: 22,
        score_b: 0,
        epoch: chrono::Utc::now(),
    }, Match {
        player_a: 1,
        player_b: 3,
        id: 0,
        ping_a: 0,
        ping_b: 0,
        rating_a: test_1.rating,
        rating_b: 1550.0,
		  deviation_a: test_1.deviation,
        deviation_b: 100.0,
	     volatility_a: test_1.volatility,
        volatility_b: DEFAULT_VOLATILITY,
        score_a: 0,
        score_b: 22,
        epoch: chrono::Utc::now(),
	 }, Match {
        player_a: 1,
        player_b: 4,
        id: 0,
        ping_a: 0,
        ping_b: 0,
        rating_a: test_1.rating,
        rating_b: 1700.0,
		  deviation_a: test_1.deviation,
        deviation_b: 300.0,
	     volatility_a: test_1.volatility,
        volatility_b: DEFAULT_VOLATILITY,
        score_a: 0,
        score_b: 22,
        epoch: chrono::Utc::now(),
	 }
    ];

	 let started = std::time::Instant::now();

	 test_1.update_player(vec_matches.clone());

	 let took = started.elapsed();

	 println!("RESULTS: ");

	 let expected_rating = 1464.06;
	 let expected_deviation = 151.52;
	 let expected_volatility = 0.05999;

	 let off_by_rating = (test_1.get_public_rating() as f64 - expected_rating).abs();
	 let off_by_deviation = (test_1.get_public_deviation() as f64 - expected_deviation).abs();
	 let off_by_volatility = (test_1.volatility - expected_volatility).abs();

    println!(
        "r': {} == {} (off by {:.2})",
			test_1.get_public_rating(),
			expected_rating,
			off_by_rating,
    );
    println!(
        "RD': {} == {} (off by {:.2})",
		  test_1.get_public_deviation(),
		  expected_deviation,
		  off_by_deviation
    );

	 println!("volatility': {} == {} (off by {:.2})", test_1.volatility, expected_volatility, off_by_volatility);

	 assert!(off_by_rating < 1.0);
	 assert!(off_by_deviation < 1.0);
	 assert!(off_by_volatility < 1.0);

	 println!("Calulations took {:?} ({} matches)", took, vec_matches.len());
}
