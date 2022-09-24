// Glicko functions to be used in calculations.rs
// Massive pain
/*

See

https://github.com/kozabrada123/Lunars/issues/9
https://en.wikipedia.org/wiki/Glicko_rating_system#Glicko-2_algorithm
http://www.glicko.net/glicko/glicko2.pdf

*/

/*
----------------------------------------------------------------

Supporting GLICKO 2.0 Functions

Includes:

g, E, v, delta

----------------------------------------------------------------

*/

use crate::db::{Match, Player};

// Processes g for Glicko.
// See https://en.wikipedia.org/wiki/Glicko_rating_system#Glicko-2_algorithm
// d here is phi in the function
fn g(d: u16) -> f64 {
    // g(Φ) = 1 / sqrt(1 + 3Φ² / π²)
    1.0 / (1.0 + (3.0 * d.pow(2) as f64 / (3.14159f64.powf(2.0)))).sqrt()
}

// Processes E(R, Rj, ϕj) for Glicko.
// See https://en.wikipedia.org/wiki/Glicko_rating_system#Glicko-2_algorithm

/*

For

Φj == Dj
μ == R
μj == Rj

exp(n) == e to the power of n

*/

fn E(R: u16, Rj: u16, Dj: u16) -> f64 {
    // E(R, Rj, Dj) = 1 / (1 + exp{-g(Φj)(R - Rj)})
    1.0 / (1.0 + 
        2.71828f64.powf(
            -g(Dj) * (R - Rj) as f64
        ))
}


// Processes v(R, D, Rj, Dj) for Glicko
// See https://en.wikipedia.org/wiki/Glicko_rating_system#Glicko-2_algorithm

/*

For

Φj == Dj

μ == R
μj == Rj

*/

fn v(R : u16, matches : Vec<Match>) -> f64 {

    let mut sum: f64 = 0.0;
    
    // Calculate the Sigma which is the sum of all v's
    for mtch in matches {
        sum += v_single(R, mtch.rank_b, mtch.deviation_b);
    }

    sum
}


// Actual calculation of v
fn v_single(R : u16, Rj: u16, Dj: u16) -> f64 {
    (g(Dj).powf(2.0) * E(R, Rj, Dj) * (1.0 - E(R, Rj, Dj))).powf(-1.0)
}


// Processes Delta for Glicko
// See https://en.wikipedia.org/wiki/Glicko_rating_system#Glicko-2_algorithm

/*

For

Φj == Dj

μ == R
μj == Rj

*/

fn delta(player : Player, matches : Vec<Match>) -> f64 {


    // First calculate the v
    let v = v(player.rank, matches.clone());


    let mut sum: f64 = 0.0;
    
    // Then calculate the Sigma which is the sum of all delta's
    for mtch in matches.clone() {
        sum += delta_single(mtch.rank_a, mtch.rank_b, mtch.deviation_b, mtch.score_b);
    }

    // Return the sum of all deltas times v
    sum * v

}

// Actual calculation of delta
/*

R, Rj, Dj, Sj are current match

matches is a vector of every match

*/
fn delta_single(R : u16, Rj: u16, Dj: u16, Sj: u8) -> f64 {
    g(Dj) * (Sj as f64 - E(R, Rj, Dj))
}


// The f(x) function, that we have to solve f(A) = 0 for
fn f(A : f64, Delta : f64, D : u16, R : u16, Sigma : f64) -> f64 {

    // r constraints the volatility over time, for instance r = 0.2 (smaller values of r prevent dramatic rating changes after upset results)
    let r: f64 = 0.2;

    0.5 * (
        (2.71828f64.powf(A) * (Delta.powf(2.0) - D.pow(2) as f64 - R.pow(2) as f64 - 2.71828f64.powf(A)))
        /
        (D.pow(2) as f64 + R as f64 + 2.71828f64.powf(A))
    )
    -
    // log(2.71828f64) should be the natural logarithm, aka the logarithm of e
    (A - (Sigma.powf(2.0).log(2.71828f64) ))
    /
    r.powf(2.0)
}

// Calculates A, which we need for the volatility
fn calc_a( Delta : f64, D : u16, R : u16, Sigma : f64) -> f64 {

    let a = 1.0;

    // TO:DO Employ the Regula Fasi Illinois algorithm

    f(a, Delta, D, R, Sigma);

    a

}