// Main file! Hosts server that connects to database.
// Make sure to first configure .env and your json file with key hashes.
// -----------------------

// Imports
// -----------------------
#[macro_use] 
extern crate nickel;
extern crate serde;
extern crate dotenv;
extern crate time;

mod calculations;
mod db;

use std::{fs};
use dotenv::dotenv;
use nickel::{Nickel, HttpRouter, JsonBody};
use nickel::status::StatusCode::{Forbidden, ImATeapot};
use serde::{Serialize, Deserialize};
use sha256::digest;
use serde_json;
// -----------------------

// Struct of the valid authentication keys
// TODO: add perms
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct AuthKey {
    hash : String,
}

/*impl AuthKey {
    fn new(hash: &str) -> AuthKey {
        AuthKey { hash: hash.to_string() }
    }
}*/

// Struct we use when we want to get data
#[derive(Serialize, Deserialize, Debug, Clone)]
struct GetStruct {
    qtype : String, // Type of data we are providing
    value : String, // Data we are providing for searching
}

// Struct of player we add
#[derive(Serialize, Deserialize, Debug, Clone)]
struct PlayerStruct {
    token : String,
    name : String,
    elo : u16,
}
// Struct of match we submit
#[derive(Serialize, Deserialize, Debug, Clone)]
struct AddStruct {
    token : String,
    user_a : String,
    ping_a : u16,
    score_a : u16,
    user_b : String,
    ping_b : u16,
    score_b : u16,
}

fn main() {
    // Make db if not exists
    db::DbConnection::new().setup();

    // Load .env file
    dotenv().ok();

    let mut server = Nickel::new();
        
    // Server paths
    server.get("/get/", middleware! {"Invalid method. Please use POST."});

    server.post("/get/player/", middleware! { |request, mut response|
        // What we'll send in response
        let mut responsedata = "".to_string();

        // Make a struct out of it
        let parameters = request.json_as::<GetStruct>().unwrap();

        match parameters.qtype.to_ascii_lowercase().as_str() {
            "id" => {

                // Connect to db
                let dbcon = db::DbConnection::new();

                // Get the player
                let player = dbcon.get_player_by_id(&parameters.value.to_owned().replace('\n', "").parse::<usize>().unwrap()).unwrap();

                // Convert player to json
                let data = serde_json::to_string(&player).unwrap();

                // Add to responsedata
                responsedata.push_str(&data.clone().to_string()); 

                dbcon.conn.close().unwrap();

            },
            "name" => {

                // Connect to db
                let dbcon = db::DbConnection::new();

                // Get the player
                let player = dbcon.get_player_by_name(&parameters.value.to_owned().replace('\n', "")).unwrap();

                // Convert player to json
                let data = serde_json::to_string(&player).unwrap();

                // Add to responsedata
                responsedata.push_str(&data.clone().to_string()); 

                dbcon.conn.close().unwrap();



            },
            "elo" => {response.set(ImATeapot); responsedata = "Oh dear god no".to_string()},
            _ => {responsedata = "Invalid qtype.".to_string()}
        }

        format!("{}", responsedata)

    });

    server.post("/get/match/", middleware! { |request, mut response|
        // What we'll send in response
        let mut responsedata = "".to_string();

        // Make a struct out of it
        let parameters = request.json_as::<GetStruct>().unwrap();

        match parameters.qtype.to_ascii_lowercase().as_str() {
            "id" => {

                // Connect to db
                let dbcon = db::DbConnection::new();

                // Get the match
                let smatch = dbcon.get_match_by_id(&parameters.value.parse::<usize>().unwrap()).unwrap();

                // Convert player to json
                let data = serde_json::to_string(&smatch).unwrap();

                // Add to responsedata
                responsedata.push_str(&data.clone().to_string()); 

                dbcon.conn.close().unwrap();

            },
            _ => {responsedata = "Invalid qtype.".to_string()}
        }

        format!("{}", responsedata)

    });

    server.get("/add/", middleware! { |_request, mut response| response.set(ImATeapot); "Invalid method. Please use POST."});

    server.post("/add/", middleware! { |request, mut response|
        // What we'll send in response
        let mut responsedata = "";

        // Authenticate with json string key
        let parameters = request.json_as::<PlayerStruct>().unwrap();
        // pass to authenticator
        let authenticated = authenticator(parameters.token);

        if !authenticated {
            // Not authenticated, get forbidden
            response.set(Forbidden);
            responsedata = "Invalid Token"
        }

        else {
            // Add player to db here

            // Connect to db
            let dbcon = db::DbConnection::new();

            // Add
            dbcon.add_player(&parameters.name.as_str(), &parameters.elo);

            dbcon.conn.close().unwrap();

        }

        format!("{}", responsedata)

    });

    server.post("/submit/", middleware! { |request, mut response|
        // What we'll send in response
        let mut responsedata = "";

        // Authenticate with json string key
        let parameters = request.json_as::<AddStruct>().unwrap();

        // pass to authenticator
        let authenticated = authenticator(parameters.token.clone());

        if !authenticated {
            // Not authenticated, get forbidden
            response.set(Forbidden);
            responsedata = "Invalid Token"
        }

        else {
            // Add game to db here
            process_game(parameters.clone());
        }

        format!("{}", responsedata)

    });

    server.listen("127.0.0.1:6767").unwrap();
}

fn authenticator(key: String) -> bool {
    // Authenticator with sha256 keys
    // TODO: Make this better somehow so we aren't saving stuff in json
    // Maybe add it into the database? Not sure how much I like that however..

    // See which file has our keys
    dotenv().ok(); // Load env
    let authfile = std::env::var("KEYFILE").unwrap();

    // Read our raw key data
    let raw_keys = fs::read_to_string(authfile)
        .expect("Couldn't read keyfile");

    // Parse it in json
    let keys: Vec<AuthKey> = serde_json::from_str(&raw_keys).unwrap();

    // Process sha256 key hash
    let key_hash = digest(key);

    // Return whether or not the know the hash
    // We use a for loop here because&we had AuthKey(key_hash) and permissions we would get false
    // Remember, we are checking if we know the key, not if we know the exact permissions thing
    // For now...
    // Eventually we'll want to check if we have the right permissions
    for auth_key in keys {
        if auth_key.hash == key_hash {return true}
    }

    false
}

// In main.rs because we can access calculations.rs from db.rs
fn process_game(data: AddStruct) {

    // Connect to db
    let dbcon = db::DbConnection::new();

    // Get both players

    let player_a = dbcon.get_player_by_name(&data.user_a).unwrap();

    let player_b = dbcon.get_player_by_name(&data.user_b).unwrap();

    // Get their elo
    let player_a_elo = player_a.elo;
    let player_b_elo = player_b.elo;

    // Calc

    let new_ranks = calculations::calculate_new_rankings(&player_a_elo, &data.ping_a, &data.score_a, &player_b_elo, &data.ping_b, &data.score_b);

    // Set new ranks
    dbcon.set_player_elo_by_id(&player_a.id, &new_ranks.0);
    dbcon.set_player_elo_by_id(&player_b.id, &new_ranks.1);

    // Calculate rank diffference
    let player_a_elo_change: i16 = player_a_elo as i16 - new_ranks.0 as i16;
    let player_b_elo_change: i16 = player_b_elo as i16 - new_ranks.1 as i16;

    // Add game to db
    dbcon.add_match(&player_a.id.try_into().unwrap(), &player_b.id.try_into().unwrap(), &data.score_a, &data.score_b, &player_a_elo_change, &player_b_elo_change);

    dbcon.conn.close().unwrap();
}