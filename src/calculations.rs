// calculations.rs: the file with all of our rank calculations.&
// explanantions for what we're doing can be found here: https://github.com/kozabrada123/Lunars/blob/main/resources/lunaro-rating-specification.pdf
// -----------------------

// Import logging
use log::{debug, info};
use std::{f64::consts::E, time::Instant};

use crate::types::entities::r#match::DebugInfo;

pub fn _test() {
    // 14-07-22
    // trying to comply with lunaro-rating-specification
    // main function which makes calls to other functions

    // what is player a's rank, ping and score?
    let rank_a: u16 = 1000;
    let ping_a: u16 = 80;
    let score_a: u8 = 20;

    // what is player b's rank, ping and score?
    let rank_b: u16 = 1000;
    let ping_b: u16 = 0;
    let score_b: u8 = 0;

    // print the inputed values for debuggings
    println!(
        "player a: rank {}, ping {}, score {}",
        rank_a, ping_a, score_a
    );

    println!(
        "player b: rank {}, ping {}, score {}",
        rank_b, ping_b, score_b
    );

    // calculate and print
    println!("calculating.. ");

    let nranks =
        calculate_new_rankings_with_elo(&rank_a, &rank_b, &ping_a, &ping_b, &score_a, &score_b);

    println!("player a's new rank: {}", nranks.0);

    println!("player b's new rank: {}", nranks.1);
}

/// Calculates the hyberbolic secant for n
pub fn sech(n: f64) -> f64 {
    let val: f64; // return value

    val = 2f64 / (E.powf(n) + E.powf(-n)); // calculate sech being 2 over e to the n + e to the -n

    val // return the calculated value
}

/// Function that calculates the ability of a player, given r, the player's rank, p, the player's ping, & i, ping influence, a preset value
/// returns a, the player's ability
///
/// here i, ping & rank are u16s as we don't expect values greater than 65535 or lower than 0
pub fn calculate_player_ability(rank: &u16, ping: &u16) -> f32 {
    let i = 300; // ping influence
    let mut ability: f32; // player ability variable we are calculating

    ability = *rank as f32 * sech(*ping as f64 / i as f64) as f32;

    // whole thing breaks if ping == 0 because (0 / 300) * rank = 0
    // so bandaid fix
    if *ping == 0 {
        ability = *rank as f32;
    }

    ability // finally, return a
}

/// Function that calculates the new rankings and returns them
/// uses rank, ping and goals of each player, with an ELO system
pub fn calculate_new_rankings_with_elo(
    rank_a: &u16,
    rank_b: &u16,
    ping_a: &u16,
    ping_b: &u16,
    goals_a: &u8,
    goals_b: &u8,
) -> (u16, u16, DebugInfo) {
    // Log for debugging
    debug!("Performing ranking calculations..");

    // For benchmarking purpuses, record current time
    let now = Instant::now();

    // first, we calculate the ability of each player
    let aa: f32 = calculate_player_ability(rank_a, ping_a); // ability of a
    let ab: f32 = calculate_player_ability(rank_b, ping_b); // ability of b

    // print for debugging..
    info!("player a's ability: {}", aa);
    info!("player b's ability: {}", ab);

    // then calculate the expected score of one player with the formula from the doc
    let ea = 1_f32 / (1_f32 + 10_f32.powf((ab - aa) / 400.0));

    // calculate the expected score ofj the other player with 1 - ea
    let eb = 1_f32 - ea;

    // print for debugging..
    info!("player a's expected score: {}", ea);
    info!("player b's expected score: {}", eb);

    // now, calculate the score of each player with the ammount of goals they scored
    let sa = *goals_a as f32 / (*goals_a as f32 + *goals_b as f32);

    let sb = 1 as f32 - sa as f32;

    // print for debugging..
    info!("player a's score: {}", sa);
    info!("player b's score: {}", sb);

    // k factor interjection
    // k is maximum rank change per game
    //
    // if rank is (0.. 1499) k = 40
    // if rank is (1500.. 2499) k = 20
    // if rank is 2500+ k = 10
    //
    // for now though k for everyone is 50
    let k = 50;

    debug!("calculating new ranks..");

    // finally: calculate the new rank for each player

    // 30-7-22 01:21 AM
    // I think we should multiply k, so that k is actually our max change per player
    // Since sa - ea give us -0.5 -- 0.5
    // ¯\_(ツ)_/¯
    // Fuck it it makes it a lot clearer
    // https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=44d0ace1438874c4c7e46c8d66fb25c9

    // 15-8-22 22:26
    // No you dumb shit I somehow fixed that
    // Reverting that above

    let n_rank_a = *rank_a as f32 + k as f32 * (sa as f32 - ea as f32);

    let n_rank_b = *rank_b as f32 + k as f32 * (sb as f32 - eb as f32);

    let elapsed = now.elapsed();

    info!(
        "finished ranking calculations, took in total {:.2?}",
        &elapsed
    );

    // return the new ranks in a tuple of u16s
    // also, give us the calculation debug info aswel
    (
        n_rank_a.round() as u16,
        n_rank_b.round() as u16,
        DebugInfo {
            time: elapsed.as_micros() as u64, // time in micro seconds
            // All other calculation fields
            ability_a: aa as u64, // convert floats here to integers, as we can forget the very small details
            ability_b: ab as u64,
            expected_a: ea,
            expected_b: eb,
            actual_a: sa,
            actual_b: sb,
        },
    )
}
