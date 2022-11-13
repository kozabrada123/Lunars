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

// Pi lmao
// we don't need 1000% precise, we only use a few decimals.
const PI: f64 = 3.14159265359;

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
fn reset_defaults(player: &mut db::Player) {
    set_rating(player, DEFAULT_RATING);
    set_Rd(player, DEFAULT_DEVIATION);
    player.volatility = DEFAULT_VOLATILITY;
}

/*
" Calculates and updates the player's rating deviation for the
beginning of a rating period. "
*/

// This function adds the volatility and deviation???
// wtf??

// Ok as long as it works
fn pre_rating_deviation(player: &mut db::Player) {
    player.deviation = (player.deviation.pow(2) as f64 + player.volatility.powf(2.0)).sqrt() as u16;
}

// Calculates the new rating and deviation for a player.
// Literally the main function we use lol

pub fn update_player(player: &mut db::Player, matches: Vec<db::Match>) {
    // Convert the values for internal use

    // First also sort the matches so player_a is always us
    let mut matches_converted = matches.clone();

    for i in 0..matches_converted.len() {
        // Go through every match and make it sorted
        matches_converted[i] = matches_converted[i].sort_by_player_id(player.id);
    }

    // Now convert the values from readable to internal
    for i in 0..matches_converted.len() {
        let mut curr_match = matches_converted[i].clone();

        // Only do the b ones since A is us
        // Convert Rank b
        curr_match.rank_b =
            (curr_match.rank_b as f64 - DEFAULT_RATING as f64 / CONVERSION_VAL) as u16;

        // Convert Deviation b
        curr_match.deviation_b = (curr_match.deviation_b as f64 / CONVERSION_VAL) as u16;

        // Save it back
    }

    // Calculate anchillary v
    let v = v(&player.clone(), matches_converted);

    // Calculate our volatility
    player.volatility = new_vol(player.clone(), matches.clone(), v);

    // Pre calculate rating deviation
    pre_rating_deviation(player);

    // Calculate our deviation
    player.deviation =
        (1.0 / ((1.0 / player.deviation.pow(2) as f64) + (1.0 / v as f64)).sqrt()) as u16;

    // Calculate our rating
    let mut tempsum = 0.0;
    for i in 0..matches.clone().len() {
        tempsum += g(&player.clone(), matches[i].clone().deviation_b)
            * (calc_match_a_score(&matches[i])
                - e(&player.clone(), matches[i].rank_b, matches[i].deviation_b));
    }

    player.rank = (player.deviation.pow(2) as f64 * tempsum) as u16;
}

// Calculates the new volatility from matches
fn new_vol(player: db::Player, matches: Vec<db::Match>, v: f64) -> f64 {
    // Step 1:
    let a = player.volatility.powf(2.0).ln();
    let eps = 0.000001;
    let mut A = a;

    // Step 2:
    let mut B: f64;
    let delta = delta(&player, matches.clone(), v.clone());
    let tau = TAU;

    if delta.powf(2.0) > (player.deviation.pow(2) as f64 + v) {
        B = (delta.powf(2.0) - player.deviation.pow(2) as f64 - v).ln();
    } else {
        let mut k = 1;
        while f(&player, a - k as f64 * tau.powf(2.0).sqrt(), delta, v, a) < 0.0 {
            k = k + 1;
        }
        B = a - k as f64 * tau.powf(2.0).sqrt();
    }

    // Step 3:
    let mut fA = f(&player, A, delta, v, a);
    let mut fB = f(&player, B, delta, v, a);

    // Step 4:
    while (B - A).abs() > eps {
        // A
        let mut C = A + ((A - B) * fA) / (fB - fA);
        let mut fC = f(&player, C, delta, v, a);

        // B
        if fC * fB < 0.0 {
            A = B;
            fA = fB;
        } else {
            fA = fA / 2.0;
        }

        // C
        B = C;
        fB = fC;
    }

    // Step 5:
    return (A / 2.0).exp();
}

// F func from glicko
fn f(player: &db::Player, x: f64, delta: f64, v: f64, a: f64) -> f64 {
    let ex = x.exp();
    let num1 = ex * (delta.powf(2.0) - player.rank.pow(2) as f64 - v - ex);
    let denom1 = 2.0 * ((player.rank.pow(2) as f64 + v + ex).powf(2.0));
    return (num1 / denom1) - ((x - a) / (TAU.powf(2.0)));
}

// The Delta func from glicko.
fn delta(player: &db::Player, matches: Vec<db::Match>, v: f64) -> f64 {
    let mut tempsum = 0.0;
    for i in 0..matches.len() {
        tempsum += g(player, matches[i].deviation_b)
            * (calc_match_a_score(&matches[i]) // Only difference here is our outcome is 0 - 1 when in glicko its 0 || 1
                - e(player, matches[i].rank_b, matches[i].deviation_b));
    }

    return v * tempsum;
}

// v func from glicko
fn v(player: &db::Player, matches: Vec<db::Match>) -> f64 {
    let mut tempsum: f64 = 0.0;
    for i in 0..matches.len() {
        let mut temp_e = e(&player.clone(), matches[i].rank_b, matches[i].deviation_b);
        tempsum += g(player, matches[i].deviation_b).powf(2.0) * temp_e * (1.0 - temp_e);
    }
    return 1.0 / tempsum;
}

// e func from glicko
fn e(player: &db::Player, rank_b: u16, deviation_b: u16) -> f64 {
    return 1.0 / (1.0 + (-1.0 * g(player, deviation_b) * (player.rank as i16 - rank_b as i16) as f64));
}

// g func from glicko
fn g(player:& db::Player, deviation: u16) -> f64 {
    return 1.0 / (1.0 + 3.0 * deviation.pow(2) as f64 / PI.powf(2.0)).sqrt();
}

// Only function not stolen and not in glicko, processes a match to a 0 - 1 float of how well player a did
fn calc_match_a_score(matchtp: &db::Match) -> f64 {
    let sa = matchtp.score_a as f64 / (matchtp.score_a as f64 + matchtp.score_b as f64);

    return sa;
}

// Applies step 6, used for players who didn't compete.
fn did_not_compete(player: &mut db::Player) {
    pre_rating_deviation(player);
}

#[test]
fn glickos_correctly() {

    let mut test_1 = db::Player {
        id: 1,
        name: "Test1".to_string(),
        deviation: 0,
        rank: 0,
        volatility: DEFAULT_VOLATILITY
    };

    set_Rd(&mut test_1, 200);
    set_rating(&mut test_1, DEFAULT_RATING);

    let vec_matches = vec![db::Match {
        player_a: 1,
        player_b: 2,
        id: 0,
        deviation_a: test_1.deviation,
        deviation_b: 30,
        ping_a: 0,
        ping_b: 0,
        rank_a: test_1.rank,
        rank_b: 1400,
        score_a: 22,
        score_b: 1,
        epoch: 0,
        volatility_a: test_1.volatility,
        volatility_b: DEFAULT_VOLATILITY
    },
    db::Match {
        player_a: 1,
        player_b: 2,
        id: 0,
        deviation_a: test_1.deviation,
        deviation_b: 100,
        ping_a: 0,
        ping_b: 0,
        rank_a: test_1.rank,
        rank_b: 1550,
        score_a: 1,
        score_b: 22,
        epoch: 0,
        volatility_a: test_1.volatility,
        volatility_b: DEFAULT_VOLATILITY
    },
    db::Match {
        player_a: 1,
        player_b: 2,
        id: 0,
        deviation_a: test_1.deviation,
        deviation_b: 300,
        ping_a: 0,
        ping_b: 0,
        rank_a: test_1.rank,
        rank_b: 1700,
        score_a: 1,
        score_b: 22,
        epoch: 0,
        volatility_a: test_1.volatility,
        volatility_b: DEFAULT_VOLATILITY
    }
    ];

    update_player(&mut test_1, vec_matches);

    println!("{} == 1464.05", test_1.rank);
}