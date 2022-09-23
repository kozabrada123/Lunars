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

g, E, v

----------------------------------------------------------------

*/

// Processes g for Glicko.
// See https://en.wikipedia.org/wiki/Glicko_rating_system#Glicko-2_algorithm
// d here is phi in the function
fn g(d: f64) -> f64 {
    // g(Φ) = 1 / sqrt(1 + 3Φ² / π²)
    1.0 / (1.0 + (3.0 * d.powf(2.0) / (3.14159f64.powf(2.0)))).sqrt()
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

fn E(R: u16, Rj: u16, Dj: f64) -> f64 {
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

fn v(R : u16, Rj: u16, Dj: f64) -> f64 {
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

fn Delta(R : u16, Rj: u16, Dj: f64, Sj : u8) -> f64 {
    v(R, Rj, Dj) * g(Dj) * (Sj as f64 - E(R, Rj, Dj))
}