// Main file! Hosts server that connects to database.
// Make sure to first configure .env and your json file with key hashes.
// -----------------------

// Imports
// -----------------------
#[macro_use]
extern crate nickel;
extern crate dotenv;
extern crate pretty_env_logger;
extern crate serde;
extern crate time;

mod calculations;
mod db;

use crate::db::{Match, Player};
use dotenv::dotenv;
use log::{debug, error, warn};
use nickel::status::StatusCode;
use nickel::{HttpRouter, JsonBody, Nickel};
use serde::{Deserialize, Serialize};
use serde_json;
use serde_json::Value;
use sha256::digest;
use std::fs;
// -----------------------

// Struct of the valid authentication keys
// TODO: add perms
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct AuthKey {
    hash: String,
}

/*impl AuthKey {
    fn new(hash: &str) -> AuthKey {
        AuthKey { hash: hunsafeash.to_string() }
    }
}*/

// Struct we used to need we wanted to get data
// Now we convert everything to just json
/*#[derive(Serialize, Deserialize, Debug, Clone)]
struct GetStruct {
    qtype : String, // Type of data we are providing
    value : String, // Data we are providing for searching
}

// Struct of player we add
#[derive(Serialize, Deserialize, Debug, Clone)]
struct PlayerStruct {
    token : String,
    name : String,
    rank : u16,
}*/
// Struct of a game
#[derive(Serialize, Deserialize, Debug, Clone)]
struct GameStruct {
    token: String,
    player_a: String,
    ping_a: u16,
    score_a: u16,
    player_b: String,
    ping_b: u16,
    score_b: u16,
}

fn main() {
    // Make db if not exists
    db::DbConnection::new().setup();

    // Load .env file
    dotenv().ok();

    // Init logger
    pretty_env_logger::init();

    // Make a nickel server
    let mut server = Nickel::new();

    // Server paths

    // Gets players
    server.get(
        "/api/players",
        middleware! { |request, mut response|

            // Log debug
            debug!("GET /api/players from {}", request.origin.remote_addr);

            // What we'll send in response
            let mut responsedata = "".to_string();

            // Connect to db
            let dbcon = db::DbConnection::new();

            // Get players
            let players = dbcon.get_players("SELECT * FROM players", &[]).unwrap();


            // Convert player to json
            let data = serde_json::to_string(&players).unwrap();

            // Add to responsedata
            responsedata.push_str(&data.clone().to_string());


            dbcon.conn.close().unwrap();

            debug!("{}: Finished request", request.origin.remote_addr);

            format!("{}", responsedata)

        },
    );

    // Gets a player
    server.get("/api/players/:query", middleware! { |request, mut response|
        // Log debug
        debug!("GET /api/players/{} from {}", request.param("query").unwrap(), request.origin.remote_addr);

        // What we'll send in response
        let mut responsedata = "".to_string();

        // Guess whether or not we got an id or name
        let query = request.param("query").unwrap();

        match query.parse::<usize>() {
            // Sucessfull parse, its a valid integer
            Ok(id) => {
                // We have an id

                // Log
                debug!("{}: Parameter is an id", request.origin.remote_addr);

                // Connect to db
                let dbcon = db::DbConnection::new();

                // Get the player
                let player: Player;

                let temp = dbcon.get_player_by_id(&id);

                //Try Check
                match &temp {
                    Ok(_res) => {
                        // Continue on normally
                        player = temp.unwrap();

                        // Convert player to json
                        let data = serde_json::to_string(&player).unwrap();

                        // Add to responsedata
                        responsedata.push_str(&data.clone().to_string());

                        // Log
                        debug!("{}: Successfully got player", request.origin.remote_addr);

                    },

                    // No errors, set custom statuscode
                    Err(rusqlite::Error::QueryReturnedNoRows) => {
                        response.set(StatusCode::NotFound);

                        responsedata = "No player was found".to_string();

                        warn!("{}: No player {} found", request.origin.remote_addr, &id);
                    },

                    // Other misc error happened
                    Err(err) => {
                        response.set(StatusCode::InternalServerError);

                        responsedata = format!("{}", err);

                        error!("{}: Misc error {} happened", request.origin.remote_addr, err);
                    }
                }

                dbcon.conn.close().unwrap();
            }

            // Unsuccessful parse, its a string that cant be parsed into a number
            Err(_err) => {
                // We have a player's name

                // Log
                debug!("{}: Parameter is a name", request.origin.remote_addr);

                // Connect to db
                let dbcon = db::DbConnection::new();

                // Get the player
                let player: Player;

                let temp = dbcon.get_player_by_name(query);

                //Try Check
                match &temp {
                    Ok(_res) => {
                        // Continue on normally
                        player = temp.unwrap();

                        // Convert player to json
                        let data = serde_json::to_string(&player).unwrap();

                        // Add to responsedata
                        responsedata.push_str(&data.clone().to_string());

                    },

                    // No errors, set custom statuscode
                    Err(rusqlite::Error::QueryReturnedNoRows) => {
                        response.set(StatusCode::NotFound);

                        responsedata = "No player was found".to_string();

                        warn!("{}: No player {} found", request.origin.remote_addr, &query);
                    },

                    // Other misc error happened
                    Err(err) => {
                        response.set(StatusCode::InternalServerError);

                        responsedata = format!("{}", err);

                        error!("{}: Misc error {} happened", request.origin.remote_addr, err);
                    }
                }

                dbcon.conn.close().unwrap();
            }
        }

        debug!("{}: Finished request", request.origin.remote_addr);

        format!("{}", responsedata)

    });

    // Gets matches
    server.get(
        "/api/matches",
        middleware! { |request, mut response|

            // Log
            debug!("GET /api/matches from {}", request.origin.remote_addr);

            // What we'll send in response
            let mut responsedata = "".to_string();

            // Connect to db
            let dbcon = db::DbConnection::new();

            // Get matches
            let matches = dbcon.get_matches("SELECT * FROM matches", &[]).unwrap();


            // Convert matches to json
            let data = serde_json::to_string(&matches).unwrap();

            // Add to responsedata
            responsedata.push_str(&data.clone().to_string());


            dbcon.conn.close().unwrap();

            debug!("{}: Finished request", request.origin.remote_addr);

            format!("{}", responsedata)

        },
    );

    // Gets a match
    server.get("/api/matches/:query", middleware! { |request, mut response|
        // Log
        debug!("GET /api/matches/{} from {}", request.param("query").unwrap(), request.origin.remote_addr);

        // What we'll send in response
        let mut responsedata = "".to_string();

        // Parse the query as it can only be an id
        let query = request.param("query").unwrap();

        let id = query.parse::<usize>().unwrap();


        // Connect to db
        let dbcon = db::DbConnection::new();

        // Get the match
        let smatch: Match;

        let temp = dbcon.get_match_by_id(&id);

        //Try Check
        match &temp {
            Ok(_res) => {
                // Continue on normally
                smatch = temp.unwrap();

                // Convert player to json
                let data = serde_json::to_string(&smatch).unwrap();

                // Add to responsedata
                responsedata.push_str(&data.clone().to_string());

                // Log
                debug!("{}: Successfully got match", request.origin.remote_addr);

            },

            // No errors, set custom statuscode
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                response.set(StatusCode::NotFound);

                responsedata = "No match was found".to_string();

                warn!("{}: No match {} found", request.origin.remote_addr, &id);
            },

            // Other misc error happened
            Err(err) => {
                response.set(StatusCode::InternalServerError);

                responsedata = format!("{}", err);

                error!("{}: Misc error {} happened", request.origin.remote_addr, err);
            }
        }

        dbcon.conn.close().unwrap();

        debug!("{}: Finished request", request.origin.remote_addr);

        format!("{}", responsedata)

    });

    /* Sets a player's name¸
    server.post("/set/player/name/", middleware! { |request, mut response|
        // What we'll send in response
        let mut responsedata = "".to_string();

        // Make a misc Value to get from
        // Expects a value like
        // "qtype":"id",
        // "value":5,
        // "name":"a"
        let parameters: Value = request.json_as().unwrap();

        // pass key to authenticator
        let authenticated = authenticator(parameters["token"].to_string().replace('"', "").clone());

        if !authenticated {
            // Not authenticated, get unathorized
            response.set(StatusCode::Unauthorized);
            responsedata = "Invalid Token".to_string()
        }

        match parameters["qtype"].to_string().replace('"', "").as_str() {
            "id" => {

                // Connect to db
                let dbcon = db::DbConnection::new();

                // Use the func
                let temp = dbcon.set_player_name_by_id(&parameters["value"].as_u64().unwrap().try_into().unwrap(), &parameters["name"].to_string().replace('"', "").as_str());

                //Try Check
                match &temp {
                    Ok(_res) => {},
                    // No errors, set custom statuscode
                    Err(rusqlite::Error::QueryReturnedNoRows) => {response.set(StatusCode::NotFound); responsedata = "No match was found".to_string();},
                    // Other misc error happened
                    Err(err) => {response.set(StatusCode::InternalServerError); responsedata = format!("{}", err);}
                }

                dbcon.conn.close().unwrap();

            },
            _ => {responsedata = "Invalid qtype.".to_string()}
        }

        response.set(StatusCode::Created);

        format!("{}", responsedata)

    });

    // Sets a player's rank
    server.post("/set/player/rank/", middleware! { |request, mut response|
            // What we'll send in response
            let mut responsedata = "".to_string();

            // Make a misc Value to get from
            // Expects a value like
            // "qtype":"id",
            // "value":5,
            // "rank":550
            let parameters: Value = request.json_as().unwrap();

            // pass key to authenticator
            let authenticated = authenticator(parameters["token"].to_string().replace('"', "").clone());

            if !authenticated {
                // Not authenticated, get unathorized
                response.set(StatusCode::Unauthorized);
                responsedata = "Invalid Token".to_string()
            }

            match parameters["qtype"].to_string().replace('"', "").as_str() {
                "id" => {

                    // Connect to db
                    let dbcon = db::DbConnection::new();

                    // Use the func
                    let temp = dbcon.set_player_rank_by_id(&parameters["value"].as_u64().unwrap().try_into().unwrap(), &parameters["rank"].as_u64().unwrap().try_into().unwrap());

                    //Try Check
                    match &temp {
                        Ok(_res) => {},
                        // No errors, set custom statuscode
                        Err(rusqlite::Error::QueryReturnedNoRows) => {response.set(StatusCode::NotFound); responsedata = "No match was found".to_string();},
                        // Other misc error happened
                        Err(err) => {response.set(StatusCode::InternalServerError); responsedata = format!("{}", err);}
                    }

                    dbcon.conn.close().unwrap();

                }
                _ => {responsedata = "Invalid qtype.".to_string()}
            }

            response.set(StatusCode::Created);

            format!("{}", responsedata)

    });*/

    // Adds a player
    server.post("/api/players/add", middleware! { |request, mut response|
        // Log debug
        debug!("POST /api/players/add from {}", request.origin.remote_addr);

        // What we'll send in response
        let mut responsedata = "";

        // Convert to json
        let parameters: Value = request.json_as().unwrap();
        // pass token to authenticator
        let authenticated = authenticator(parameters["token"].to_string());

        if !authenticated {
            // Not authenticated, get unathorized
            response.set(StatusCode::Unauthorized);
            responsedata = "Invalid Token";

            // Log
            warn!("{}: Rejected, Invalid Token", request.origin.remote_addr);
        }

        else {
            // Add player to db here

            // Connect to db
            let dbcon = db::DbConnection::new();

            // Add
            dbcon.add_player(
                &parameters["name"].to_string().replace('"', "").as_str(),
                &parameters["rank"].as_u64().unwrap().try_into().unwrap()
            );

            dbcon.conn.close().unwrap();

            // Log
            debug!(
                "{}: Added player {} to database",
                request.origin.remote_addr, &parameters["name"].to_string().replace('"', "").as_str()
            );

            response.set(StatusCode::Created);
        }


        // Log
        debug!("{}: Finished request", request.origin.remote_addr);

        format!("{}", responsedata)

    });

    // Submits a match
    server.post("/api/matches/add", middleware! { |request, mut response|
        // Log debug
        debug!("POST /api/matches/add from {}", request.origin.remote_addr);

        // What we'll send in response
        let mut responsedata = "".to_string();

        // Convert to json
        let parameters: GameStruct = request.json_as().unwrap();

        // pass key to authenticator
        let authenticated = authenticator(parameters.token.clone());

        if !authenticated {
            // Not authenticated, get unathorized
            response.set(StatusCode::Unauthorized);
            responsedata = "Invalid Token".to_string();

            // Log
            warn!("{}: Rejected, Invalid Token", request.origin.remote_addr);
        }

        else {
            // Just keep track of if we have both players and valid
            let mut valid = true;

            // Get both players
            // Connect to db
            let dbcon = db::DbConnection::new();

            let mut player_a: &Player = &Player {id: 0, name: "None".to_string(), rank: 1000};
            let mut player_b: &Player = &Player {id :0, name:"None".to_string(), rank:1000};

            // Try to get players
            // A
            let temp_a = dbcon.get_player_by_name(&parameters.player_a);

            match &temp_a {
                Ok(player) => player_a = player,

                // No errors, set custom statuscode
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    response.set(StatusCode::BadRequest);

                    responsedata = parameters.player_a.clone();

                    responsedata.push_str(" not a valid player");

                    valid = false;

                    warn!(
                        "{}: No player {} found (player_a)",
                        request.origin.remote_addr,
                        &parameters.player_a
                    );
                },

                // Other misc error happened
                Err(err) => {
                    response.set(StatusCode::InternalServerError);

                    responsedata = "Internal Server Error".to_string();

                    valid = false;

                    error!("{}: Misc error {} happened", request.origin.remote_addr, err);
                }
            }

            // B
            let temp_b = dbcon.get_player_by_name(&parameters.player_a);

            match &temp_b {
                Ok(player) => player_b = player,

                // No errors, set custom statuscode
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    response.set(StatusCode::BadRequest);

                    responsedata = parameters.player_a.clone();

                    responsedata.push_str(" not a valid player");

                    valid = false;

                    warn!(
                        "{}: No player {} found (player_b)",
                        request.origin.remote_addr,
                        &parameters.player_a
                    );
                },

                // Other misc error happened
                Err(err) => {
                    response.set(StatusCode::InternalServerError);

                    responsedata = "Internal Server Error".to_string();

                    valid = false;

                    error!("{}: Misc error {} happened", request.origin.remote_addr, err);
                }
            }

            if valid {
                // If we're still "valid"
                // Process the game
                process_game(parameters.clone(), player_a, player_b);

                response.set(StatusCode::Created);

                // Log
                debug!("{}: Successfully created match", request.origin.remote_addr);
            }


        }

        // Log
        debug!("{}: Finished request", request.origin.remote_addr);

        format!("{}", responsedata)

    });

    server.listen("0.0.0.0:6767").unwrap();
}

fn authenticator(ikey: String) -> bool {
    // Authenticator with sha256 keys
    // TODO: Make this better somehow so we aren't saving stuff in json
    // Maybe add it into the database? Not sure how much I like that however..

    let mut key = ikey;

    // Process key
    if key.contains(r#"""#) {
        key = key.replace(r#"""#, "");
    }

    // See which file has our keys
    dotenv().ok(); // Load env
    let authfile = std::env::var("KEYFILE").unwrap();

    // Read our raw key data
    let raw_keys = fs::read_to_string(authfile).expect("Couldn't read keyfile");

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
        if auth_key.hash == key_hash {
            return true;
        }
    }

    false
}

// In main.rs because we can access calculations.rs from db.rs
fn process_game(data: GameStruct, player_a: &Player, player_b: &Player) {
    // Connect to db
    let dbcon = db::DbConnection::new();

    // Get thei players' rank
    let player_a_rank = player_a.rank;
    let player_b_rank = player_b.rank;

    // Calc

    let new_ranks = calculations::calculate_new_rankings(
        &player_a_rank,
        &data.ping_a,
        &data.score_a,
        &player_b_rank,
        &data.ping_b,
        &data.score_b,
    );

    // Set new ranks
    dbcon
        .set_player_rank_by_id(&player_a.id, &new_ranks.0)
        .unwrap();
    dbcon
        .set_player_rank_by_id(&player_b.id, &new_ranks.1)
        .unwrap();

    // Calculate rank diffference
    let a_delta: i16 = player_a_rank as i16 - new_ranks.0 as i16;
    let b_delta: i16 = player_b_rank as i16 - new_ranks.1 as i16;

    // Add game to db
    dbcon.add_match(
        &player_a.id.try_into().unwrap(),
        &player_b.id.try_into().unwrap(),
        &data.score_a,
        &data.score_b,
        &a_delta,
        &b_delta,
    );

    dbcon.conn.close().unwrap();
}

// Secondary method used for tests
#[allow(dead_code)]
fn process_game_test(data: GameStruct, player_a: &Player, player_b: &Player) {
    // Connect to db
    let dbcon = db::DbConnection::new_named("/tmp/randomdb.sqlite");

    // Get thei players' rank
    let player_a_rank = player_a.rank;
    let player_b_rank = player_b.rank;

    // Calc

    let new_ranks = calculations::calculate_new_rankings(
        &player_a_rank,
        &data.ping_a,
        &data.score_a,
        &player_b_rank,
        &data.ping_b,
        &data.score_b,
    );

    // Set new ranks
    dbcon
        .set_player_rank_by_id(&player_a.id, &new_ranks.0)
        .unwrap();
    dbcon
        .set_player_rank_by_id(&player_b.id, &new_ranks.1)
        .unwrap();

    // Calculate rank diffference
    let a_delta: i16 = player_a_rank as i16 - new_ranks.0 as i16;
    let b_delta: i16 = player_b_rank as i16 - new_ranks.1 as i16;

    // Add game to db
    dbcon.add_match(
        &player_a.id.try_into().unwrap(),
        &player_b.id.try_into().unwrap(),
        &data.score_a,
        &data.score_b,
        &a_delta,
        &b_delta,
    );

    dbcon.conn.close().unwrap();
}

// Tests!
#[test]
fn cant_get_none_player() {
    let result =
        db::DbConnection::new_named("/tmp/randomdb.sqlite").get_player_by_name("i_do n t exist");
    assert!(result.is_err());
}

#[test]
fn player_names_arent_case_sensitive() {
    let mut dbcon = db::DbConnection::new_named("/tmp/randomdb.sqlite");

    dbcon.setup();
    dbcon.add_player(&"IAmNotCaseSensitive", &1000);

    let a = dbcon.get_player_by_name("IAmNotCaseSensitive").unwrap();

    let b = dbcon.get_player_by_name("iamnotcasesensitive").unwrap();

    assert!(a == b);
}

#[test]
fn no_sql_injection() {
    let mut dbcon = db::DbConnection::new_named("/tmp/randomdb.sqlite");

    dbcon.setup();
    dbcon.add_player(&"Robert'); DROP TABLE players;", &1000);

    let a = dbcon.get_player_by_name("RobertDROPTABLEPLAYERS");

    let b = dbcon.get_player_by_name("Robert'); DROP TABLE players;");

    assert!(a.is_ok() && b.is_ok() && (a == b));
}

#[test]
fn can_process_games() {
    // Test whether or not we can process a game and correctly calculate the scores

    let mut dbcon = db::DbConnection::new_named("/tmp/randomdb.sqlite");

    dbcon.setup();

    // Add players
    dbcon.add_player(&"p1", &1000);
    dbcon.add_player(&"p2", &1000);

    // Make up data
    let data = GameStruct {
        token: "None".to_string(),
        player_a: "p1".to_string(),
        ping_a: 50,
        score_a: 10,
        player_b: "p2".to_string(),
        ping_b: 0,
        score_b: 1,
    };

    // Get players
    let player_a = dbcon.get_player_by_name("p1").unwrap();
    let player_b = dbcon.get_player_by_name("p2").unwrap();

    // Call process game
    process_game_test(data, &player_a, &player_b);

    // See if we succesfully calculated everything
    let a = dbcon.get_player_by_name("p1").unwrap();

    let b = dbcon.get_player_by_name("p2").unwrap();

    // Assert whether or not the scores are changed
    // https://play.rust-lang.org/?version=stable&mode=debug&edition=2021&gist=b30ef0f1ff39af4f57f5ae16b1a374e8

    assert!(a.rank != player_a.rank && b.rank != player_b.rank)
}
