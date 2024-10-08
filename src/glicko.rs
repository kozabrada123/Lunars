//! Internal ranking system logic

// Huge thanks to https://github.com/deepy/glicko2/blob/master/glicko2/glicko2.py
// they made this implementation wayyy easier

// Another huge thanks to @gpluscb for their "So You Want To Use Glicko-2 For Your Game's Ratings"
// gist and instant-glicko-2 crate.
//
// gist: https://gist.github.com/gpluscb/302d6b71a8d0fe9f4350d45bc828f802
// crate: https://github.com/gpluscb/instant-glicko-2

use std::f64::consts::PI;

use rocket::form::validate::Len;

use crate::{
    calculations::sech,
    types::entities::{player::*, r#match::*},
};

/// Duration of a "season" or rating period for the system.
pub const RATING_PERIOD_DURATION: chrono::Duration = chrono::TimeDelta::weeks(3);

pub fn rating_period_duration_days() -> u64 {
    RATING_PERIOD_DURATION.num_days() as u64
}

/// System's tau, constains the volatility change over time.
pub const TAU: f64 = 0.5;

pub fn tau() -> f64 {
    TAU
}

/// How much we inflate the value for our users.
// The python thing had 173.7178
const RATING_CONVERSION_CONSTANT: f64 = 173.7178;

/// Note: from the old ping compensation formulas
pub const PING_INFLUENCE: f64 = 300.0;

pub fn ping_influence() -> f64 {
    PING_INFLUENCE
}

// Please dont touch these!!
// These are the set values a player gets when joining the system.
pub const DEFAULT_RATING: u16 = 1500;
pub const DEFAULT_DEVIATION: u16 = 350;
pub const DEFAULT_VOLATILITY: f64 = 0.06;

pub fn default_rating() -> f64 {
    DEFAULT_RATING as f64
}

pub fn default_deviation() -> f64 {
    DEFAULT_DEVIATION as f64
}

pub fn default_volatility() -> f64 {
    DEFAULT_VOLATILITY
}

/// Function that normalizes a player's rating for showing
pub fn rating_to_public(rating: f64) -> f64 {
    (rating as f64 * RATING_CONVERSION_CONSTANT) + DEFAULT_RATING as f64
}

/// Function that normalizes a player's deviation for showing
pub fn deviation_to_public(deviation: f64) -> f64 {
    deviation as f64 * RATING_CONVERSION_CONSTANT
}

/// Function that un-normalizes a player's rating
pub fn rating_from_public(public_rating: f64) -> f64 {
    (public_rating as f64 - DEFAULT_RATING as f64) as f64 / RATING_CONVERSION_CONSTANT
}

/// Function that un-normalizes a player's deviation
pub fn deviation_from_public(deviation: f64) -> f64 {
    deviation as f64 / RATING_CONVERSION_CONSTANT
}

impl Player {
    /// Function that gets a player's rating for calculation
    pub fn get_private_rating(&self) -> f64 {
        rating_from_public(self.rating)
    }

    /// Sets a rating for a player
    ///
    /// uses private values
    pub fn set_private_rating(&mut self, new_rating: f64) {
        self.rating = rating_from_public(new_rating);
    }

    /// Same as [Self::get_private_rating], normalizes the deviation
    pub fn get_private_deviation(&self) -> f64 {
        deviation_from_public(self.deviation)
    }

    /// Same as set rating, sets the value from the normalized
    pub fn set_private_deviation(&mut self, new_deviation: f64) {
        self.deviation = deviation_from_public(new_deviation);
    }

    /// Function that gets a player's rating for showing
    pub fn get_public_rating(&self) -> f64 {
        self.rating
    }

    /// Sets a rating for a player
    ///
    /// uses normalized / public values
    pub fn set_public_rating(&mut self, new_rating: f64) {
        self.rating = new_rating;
    }

    /// Same as [Self::get_public_rating], normalizes the deviation
    pub fn get_public_deviation(&self) -> f64 {
        self.deviation
    }

    /// Same as set rating, sets the value from the normalized
    pub fn set_public_deviation(&mut self, new_deviation: f64) {
        self.deviation = new_deviation;
    }

    /// Resets / sets a player to default stats
    pub fn reset_defaults(&mut self) {
        self.set_public_rating(DEFAULT_RATING as f64);
        self.set_public_deviation(DEFAULT_DEVIATION as f64);
        self.volatility = DEFAULT_VOLATILITY;
    }

    /// "Calculates and updates the player's rating deviation for the
    /// beginning of a rating period."
    ///
    /// This is the second main modification of glicko-2 and
    /// where the magic of lichess and instant-glicko-2 come in.
    ///
    /// We can apply fractional rating periods, to find a rating deviation difference before
    /// the end of this rating period.
    ///
    /// Thank you for all your work gpluscb!
    fn apply_pre_rating_deviation(&mut self, elapsed_periods: f64) {
        self.deviation =
            (self.deviation.powi(2) + elapsed_periods * self.volatility.powi(2)).sqrt();
    }

    /// Calculates and updates the new rating+friends for a player.
    pub fn rate_player_for_elapsed_periods(
        &mut self,
        input_matches: Vec<Match>,
        elapsed_periods: f64,
    ) {
        // Only while we're calculating, make the inner values the private ones
        self.rating = self.get_private_rating();
        self.deviation = self.get_private_deviation();

        // If matches are empty, only apply step 6
        if input_matches.is_empty() {
            self.apply_pre_rating_deviation(elapsed_periods);

            self.rating = rating_to_public(self.rating);
            self.deviation = deviation_to_public(self.deviation);
            return;
        }

        // Convert the values for internal use

        // First also sort the matches so player_a is always us
        let mut matches = Vec::new();

        for game_match in input_matches {
            matches.push(game_match.sorted_by_player_id(self.id));
        }

        // See issue #13 - concatinate similar matches into one matchup
        //
        // another change we made to the core glicko algorithm
        //
        // Make sure similar matches are together in the array
        matches.sort_by(|match_a, match_b| {
            match_a
                .player_b
                .cmp(&match_b.player_b)
                .then_with(|| match_a.epoch.cmp(&match_b.epoch))
        });

        // Merge similar ones using an iterative algorithm
        //
        // Iterate through the matches, merge two if they are similar, repeat until all are merged
        let mut merged_this_iteration = 1;
        while merged_this_iteration > 0 {
            merged_this_iteration = 0;
            let mut merged_matches = Vec::new();

            // Iterate through merged matches
            let mut match_index = 0;
            while match_index < matches.len() {
                let mut match_a = matches[match_index].clone();
                let match_b_res = matches.get(match_index + 1).clone();

                if let Some(match_b) = match_b_res {
                    let against_same_player = match_a.player_b == match_b.player_b;

                    let at_similar_time = (match_a.epoch - match_b.epoch).abs().num_hours() <= 4;

                    let similar_ping = {
                        let same_ping = (match_a.ping_a == match_b.ping_a)
                            && (match_a.ping_b == match_b.ping_b);

                        let similar_ping_ability_for_player_a =
                            (calculate_ping_ability(match_a.ping_a)
                                - calculate_ping_ability(match_b.ping_a))
                            .abs()
                                <= 0.1;
                        let similar_ping_ability_for_player_b =
                            (calculate_ping_ability(match_a.ping_b)
                                - calculate_ping_ability(match_b.ping_b))
                            .abs()
                                <= 0.1;

                        let similar_ping_abilities =
                            similar_ping_ability_for_player_a && similar_ping_ability_for_player_b;

                        // I hope the compiler optimizes this and only calculates the above variables if
                        // same ping is false
                        same_ping || similar_ping_abilities
                    };

                    if against_same_player && at_similar_time && similar_ping {
                        // Concatinate both matches into match a
                        match_a.score_a += match_b.score_a;
                        match_a.score_b += match_b.score_b;

                        // FIXME: taking the average here is probably not a perfect solution
                        //
                        // Usually when you play multiple matches against someone you will
                        // have a very similar ping though, you may even report the same
                        match_a.ping_a = (match_a.ping_a + match_b.ping_a) / 2;
                        match_a.ping_b = (match_a.ping_b + match_b.ping_b) / 2;

                        merged_matches.push(match_a);

                        merged_this_iteration += 1;

                        // Skip the match we merged into a and go onto the next one
                        match_index += 2;
                        continue;
                    }
                }

                // Do not merge, keep the same match
                merged_matches.push(match_a);

                match_index += 1;
            }

            matches = merged_matches;
        }

        // Now convert the values from readable to internal
        for game_match in matches.iter_mut() {
            // Only convert b values since a is us, and we'll use our current rating
            game_match.rating_b = rating_from_public(game_match.rating_b);
            game_match.deviation_b = deviation_from_public(game_match.deviation_b);
        }

        // Step 3: Calculate anchillary variance
        let variance = calculate_variance(&self, &matches);

        // Step 4 and 5: Calculate volatility with delta
        self.volatility = calculate_volatility(&self, &matches, variance);

        // Step 6
        self.apply_pre_rating_deviation(elapsed_periods);

        // Step 7: Calculate our deviation
        self.deviation = 1.0 / ((1.0 / self.deviation.powi(2)) + (1.0 / variance)).sqrt();

        // Calculate our rating
        let mut temp_sum = 0.0;

        for game_match in matches {
            temp_sum += calculate_g(game_match.deviation_b)
                * (calculate_match_a_score(&game_match)
                    - calculate_e(
                        &self,
                        game_match.ping_a,
                        game_match.rating_b,
                        game_match.deviation_b,
                        game_match.ping_b,
                    ));
        }

        self.rating += self.deviation.powi(2) * temp_sum;

        // Reset back to public ones
        self.rating = rating_to_public(self.rating);
        self.deviation = deviation_to_public(self.deviation);
    }
}

/// Calculates the new volatility from matches
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

/// F func from glicko
fn calculate_f(player: &Player, x: f64, delta: f64, v: f64, a: f64) -> f64 {
    let ex = x.exp();

    let num1 = ex * (delta.powi(2) - player.rating.powi(2) - v - ex);

    let denom1 = 2.0 * ((player.rating.powi(2) + v + ex).powi(2));

    (num1 / denom1) - ((x - a) / (TAU.powi(2)))
}

/// The Delta func from glicko.
fn calculate_delta(player: &Player, matches: &Vec<Match>, v: f64) -> f64 {
    let mut temp_sum = 0.0;
    for game_match in matches {
        temp_sum += calculate_g(game_match.deviation_b)
            * (calculate_match_a_score(game_match) // Only difference here is our outcome is 0 - 1 when in glicko its 0 || 1
                - calculate_e(player, game_match.ping_a, game_match.rating_b, game_match.deviation_b, game_match.ping_b));
    }

    return v * temp_sum;
}

/// v func from glicko
fn calculate_variance(player: &Player, matches: &Vec<Match>) -> f64 {
    let mut temp_sum: f64 = 0.0;

    for game_match in matches {
        let temp_e = calculate_e(
            &player,
            game_match.ping_a,
            game_match.rating_b,
            game_match.deviation_b,
            game_match.ping_b,
        );
        temp_sum += calculate_g(game_match.deviation_b).powi(2) * temp_e * (1.0 - temp_e);
    }

    1.0 / temp_sum
}

/// e func from glicko
///
/// This has been modifies to use the player's abilities instead of their ratings, to hopefully
/// imitate the old elo based system with regards of ping.
///
/// If we observe E in glicko 1.0, it is incredibly similar to the calculation of expected scores,
/// which is where we targeted ping compensation in elo.
fn calculate_e(player: &Player, ping_a: u16, rating_b: f64, deviation_b: f64, ping_b: u16) -> f64 {
    let ability_a = calculate_player_ability_for_glicko(player.rating, ping_a);
    let ability_b = calculate_player_ability_for_glicko(rating_b, ping_b);

    1.0 / (1.0 + (-calculate_g(deviation_b) * (ability_a - ability_b)).exp())
    // ORRR potentially put the ping compensation logic ↑ here;
    //
    // Instead of rating_a - rating_b it could be ability a - ability b
    //
    // Later note: this is indeed what I did
}

/// g func from glicko
fn calculate_g(deviation: f64) -> f64 {
    1.0 / (1.0 + 3.0 * deviation.powi(2) / PI.powi(2)).sqrt()
}

/// One function not stolen and not in glicko, processes a match to a 0 - 1 float of how well player a did
fn calculate_match_a_score(game_match: &Match) -> f64 {
    game_match.score_a as f64 / (game_match.score_a + game_match.score_b) as f64
}

/// One function not stolen and not in glicko, calculates the ability of a player, given r, the player's rating, p, the player's ping, & i, ping influence, a preset value
/// returns a, the player's ability
///
/// This is reminiscent of the player ability calculation in the old version of the rating system.
pub fn calculate_player_ability_for_glicko(rating: f64, ping: u16) -> f64 {
    // whole thing breaks if ping == 0 because (0 / 300) * rating = 0
    // so bandaid fix
    if ping == 0 {
        return rating;
    }

    // Note: this is quite jank; this is done because in glicko internal
    // math rating is somehow centered on the default.
    //
    // We want to scale with the "readable" rating; more ping makes you play worse,
    // not more like the average player.
    let normalized_rating = rating_to_public(rating);

    let normalized_ability = normalized_rating * calculate_ping_ability(ping);

    rating_from_public(normalized_ability)
}

/// Calculates the expected ability for a ping value.
///
/// Outputs a float between 0 and 1;
///
/// 1 means ping does not influence ability, 0 means the player basically cannot play
pub fn calculate_ping_ability(ping: u16) -> f64 {
    sech(ping as f64 / PING_INFLUENCE)
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

    test_1.set_public_rating(1500.0);
    test_1.set_public_deviation(200.0);

    println!("Starting r': {}", test_1.get_public_rating());
    println!("Starting RD': {}", test_1.get_public_deviation());

    let vec_matches = vec![
        Match {
            rating_period: 0,
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
        },
        Match {
            rating_period: 0,
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
        },
        Match {
            rating_period: 0,
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
        },
    ];

    let started = std::time::Instant::now();

    test_1.rate_player_for_elapsed_periods(vec_matches.clone(), 1.0);

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

    println!(
        "volatility': {} == {} (off by {:.2})",
        test_1.volatility, expected_volatility, off_by_volatility
    );

    assert!(off_by_rating < 1.0);
    assert!(off_by_deviation < 1.0);
    assert!(off_by_volatility < 1.0);

    println!(
        "Calulations took {:?} ({} matches)",
        took,
        vec_matches.len()
    );
}
