/*

Contains our middlewares that were in main.rs
Refactored though to be better

All of them modify response and return a responsedata string
*/

use crate::calculations;
use crate::db::{self, *};
use dotenv::dotenv;
use log::{debug, error, warn, info};
use nickel::status::StatusCode;
use nickel::{JsonBody};
use serde::{Deserialize, Serialize};
use serde_json;
use sha256::digest;
use std::{fs};

use nickel::{Request, Response};

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

/* Struct we used to need we wanted to get data
// Now we convert everything to just json
#[derive(Serialize, Deserialize, Debug, Clone)]
struct GetStruct {
    qtype : String, // Type of data we are providing
    value : String, // Data we are providing for searching
}
*/
// Struct of player we add
#[derive(Serialize, Deserialize, Debug, Clone)]
struct AddPlayerStruct {
    token : String,
    name : String,
    rank : u16,
}
// Struct of a game
#[derive(Serialize, Deserialize, Debug, Clone)]
struct GameStruct {
    token: String,
    player_a: String,
    player_b: String,
    ping_a: u16,
    ping_b: u16,
    score_a: u8,
    score_b: u8,
}

// Struct for dummy game
// Eh could be written better but idk
// with it being one struct
// todo refactor and merge GameStruct and DummyGameStruct
#[derive(Serialize, Deserialize, Debug, Clone)]
struct DummyGameStruct {
    // Same as GameStruct, but no token needed
    player_a: String,
    player_b: String,
    ping_a: u16,
    ping_b: u16,
    score_a: u8,
    score_b: u8,
}

// -----------------------
// GET REQUESTS

// /api/players
pub fn get_players(request: &mut Request, response: &mut Response) -> String {
    // Log debug
    debug!("GET /api/players from {}", request.origin.remote_addr);

    // What we'll send in response
    let mut responsedata = "".to_string();

    // Connect to db
    let dbcon = db::DbConnection::new();

    // Get players and see if they exist
    let players = dbcon.get_players(
        &build_query("SELECT * FROM players".to_string(), request, false), // Call build query now to process ?max and ?min and others
        &[]
    );

    // Make a blank data that we'll change if we actually have any
    let mut data = "[]".to_string();

    match players {
        Ok(playervec) => {
            // We got actual players, add them to data
            data = serde_json::to_string(&playervec).unwrap();
        },
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // We didn't get any player for this query, let data be empty
        }

        // Other misc error happened
        Err(err) => {
            response.set(StatusCode::InternalServerError);

            error!(
                "{}: Misc error {} happened",
                request.origin.remote_addr, err
            );

            return format!("{}", err);
        }
    }

    // Add to responsedata
    responsedata.push_str(&data.clone().to_string());
    response.set(nickel::MediaType::Json);

    dbcon.conn.close().unwrap();

    debug!("{}: Finished request", request.origin.remote_addr);

    return responsedata;
}

// /api/players/search/:query
pub fn get_player_search(request: &mut Request, response: &mut Response) -> String {
    // Log debug
    debug!("GET /api/search/{} from {}", request.param("query").unwrap(), request.origin.remote_addr);

    // What we'll send in response
    let mut responsedata = "".to_string();

    // Connect to db
    let dbcon = db::DbConnection::new();

    // Get players and see if they exist
    let players = dbcon.get_players(
        &build_query(format!("SELECT * FROM players WHERE name LIKE '%{}%'", sanitise(request.param("query").unwrap())), request, true), // Call build query now to process ?max and ?min and others
        &[]
    );

    // Make a blank data that we'll change if we actually have any
    let mut data = "[]".to_string();

    match players {
        Ok(playervec) => {
            // We got actual players, add them to data
            data = serde_json::to_string(&playervec).unwrap();
        },
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // We didn't get any player for this query, let data be empty
        }

        // Other misc error happened
        Err(err) => {
            response.set(StatusCode::InternalServerError);

            error!(
                "{}: Misc error {} happened",
                request.origin.remote_addr, err
            );

            return format!("{}", err);
        }
    }

    // Add to responsedata
    responsedata.push_str(&data.clone().to_string());
    response.set(nickel::MediaType::Json);

    dbcon.conn.close().unwrap();

    debug!("{}: Finished request", request.origin.remote_addr);

    return responsedata;
}

// /api/players/:query
pub fn get_player(request: &mut Request, response: &mut Response) -> String {
    // Log debug
    debug!(
        "GET /api/players/{} from {}",
        request.param("query").unwrap(),
        request.origin.remote_addr
    );

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
                Ok(_res) => {}

                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    response.set(StatusCode::NotFound);

                    warn!("{}: No player {} found", request.origin.remote_addr, &id);

                    return format!("No player {} found", &id);
                }

                // Other misc error happened
                Err(err) => {
                    response.set(StatusCode::InternalServerError);

                    error!(
                        "{}: Misc error {} happened",
                        request.origin.remote_addr, err
                    );

                    return format!("{}", err);
                }
            }

            // Continue on normally
            player = temp.unwrap();

            // Convert player to json
            let data = serde_json::to_string(&player).unwrap();

            // Add to responsedata
            responsedata.push_str(&data.clone().to_string());
            response.set(nickel::MediaType::Json);

            // Log
            debug!("{}: Successfully got player", request.origin.remote_addr);

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
                Ok(_res) => {}

                // No errors, set custom statuscode
                Err(rusqlite::Error::QueryReturnedNoRows) => {
                    response.set(StatusCode::NotFound);

                    warn!("{}: No player {} found", request.origin.remote_addr, &query);

                    return "No player was found".to_string();
                }

                // Other misc error happened
                Err(err) => {
                    response.set(StatusCode::InternalServerError);

                    error!(
                        "{}: Misc error {} happened",
                        request.origin.remote_addr, err
                    );

                    return format!("{}", err);
                }
            }

            // Continue on normally
            player = temp.unwrap();

            // Convert player to json
            let data = serde_json::to_string(&player).unwrap();

            // Add to responsedata
            responsedata.push_str(&data.clone().to_string());
            response.set(nickel::MediaType::Json);

            dbcon.conn.close().unwrap();
        }
    }

    debug!("{}: Finished request", request.origin.remote_addr);

    return responsedata;
}

// /api/matches
pub fn get_matches(request: &mut Request, response: &mut Response) -> String {
    // Log
    debug!("GET /api/matches from {}", request.origin.remote_addr);

    // What we'll send in response
    let mut responsedata = "".to_string();

    // Connect to db
    let dbcon = db::DbConnection::new();
    
    // Get matches and see if they exist
    let matches = dbcon.get_matches(
        &build_query("SELECT * FROM matches".to_string(), request, false), // Call build query now to process url params and others
        &[]
    );

    // Make a blank data that we'll change if we actually have any
    let mut data = "[]".to_string();

    match matches {
        Ok(matchesvec) => {
            // We got actual matches, add them to data
            data = serde_json::to_string(&matchesvec).unwrap();
        },
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // We didn't get any matches for this query, let data be empty
        }

        // Other misc error happened
        Err(err) => {
            response.set(StatusCode::InternalServerError);

            error!(
                "{}: Misc error {} happened",
                request.origin.remote_addr, err
            );

            return format!("{}", err);
        }
    }

    // Add to responsedata
    responsedata.push_str(&data.clone().to_string());
    response.set(nickel::MediaType::Json);

    dbcon.conn.close().unwrap();

    debug!("{}: Finished request", request.origin.remote_addr);

    return responsedata;
}

// /api/matches/:query
pub fn get_match(request: &mut Request, response: &mut Response) -> String {
    // Log
    debug!(
        "GET /api/matches/{} from {}",
        request.param("query").unwrap(),
        request.origin.remote_addr
    );

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
        Ok(_res) => {}

        // No errors, set custom statuscode
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            response.set(StatusCode::NotFound);

            warn!("{}: No match {} found", request.origin.remote_addr, &id);

            return "No match was found".to_string();
        }

        // Other misc error happened
        Err(err) => {
            response.set(StatusCode::InternalServerError);

            error!("{}: Misc error {} occured", request.origin.remote_addr, err);

            return format!("Misc error {} occured", err);
        }
    }

    // Continue on normally
    smatch = temp.unwrap();

    // Convert player to json
    let data = serde_json::to_string(&smatch).unwrap();

    // Add to responsedata
    responsedata.push_str(&data.clone().to_string());
    response.set(nickel::MediaType::Json);

    // Log
    debug!("{}: Successfully got match", request.origin.remote_addr);

    dbcon.conn.close().unwrap();

    debug!("{}: Finished request", request.origin.remote_addr);

    return responsedata;
}

// -----------------------
// POST REQUESTS

// /api/players/add
pub fn add_player(request: &mut Request, response: &mut Response) -> String {
    // Log debug
    debug!("POST /api/players/add from {}", request.origin.remote_addr);

    // Convert to json
    let parameters: AddPlayerStruct;

    // Lets see if everything is good
    match request.json_as() {
        Ok(params) => {
            // We parsed it properly, hooray!
            parameters = params;
        },
        Err(err) => {
            // Ya fucked up boy

            warn!(
                "{}: Bad Request, {}",
                request.origin.remote_addr, err
            );
            
            response.set(StatusCode::BadRequest);
            return err.to_string();
        }
    }

    // pass token to authenticator
    let authenticated = authenticator(parameters.token.to_string());

    if !authenticated {
        // Not authenticated, get unathorized
        response.set(StatusCode::Unauthorized);

        // Log
        warn!("{}: Rejected, Invalid Token", request.origin.remote_addr);

        return "Invalid Token".to_string();
    }

    // Connect to db
    let dbcon = db::DbConnection::new();

    // Check if the user already exists
    match dbcon.get_player_by_name(parameters.name.to_string().replace('"', "").as_str()) {
        Ok(_player) => {
            // Already exists
            response.set(StatusCode::Conflict);
            return "Player already exists!".to_string();
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            // No player exists..
        }
        Err(err) => {
            // Unknown error
            response.set(StatusCode::InternalServerError);

            error!("{}: Misc error {} occured", request.origin.remote_addr, err);

            return format!("Misc error {} occured", err);
        }
    }

    // Add
    dbcon.add_player(
        &parameters.name.to_string().replace('"', "").as_str(),
        &parameters.rank,
    );

    // Get the created player's id from cursor.last_insert_rowid()
    // We need to return id
    // Technically this could fail in some cases if we processed another request in the few μs but I'll fix it when it comes to that
    let rid = dbcon.conn.last_insert_rowid();

    // Get the created player from their id
    let rplayer = dbcon.get_player_by_id(&rid.try_into().unwrap()).unwrap();

    // Safe to close the connection now
    dbcon.conn.close().unwrap();

    // Log
    debug!(
        "{}: Added player {} to database",
        request.origin.remote_addr,
        &parameters.name.to_string().replace('"', "").as_str()
    );

    response.set(nickel::MediaType::Json);
    response.set(StatusCode::Created);

    // Log
    debug!("{}: Finished request", request.origin.remote_addr);

    return serde_json::to_string(&rplayer).unwrap()
}

// /api/matches/add
pub fn add_match(request: &mut Request, response: &mut Response) -> String {
    // Log debug
    debug!("POST /api/matches/add from {}", request.origin.remote_addr);

    // Convert to json
    // See if its valid
    let parameters: GameStruct;

    // Lets see if everything is good
    match request.json_as() {
        Ok(params) => {
            // We parsed it properly, hooray!
            parameters = params;
        },
        Err(err) => {
            // Ya fucked up boy
            warn!(
                "{}: Bad Request, {}",
                request.origin.remote_addr, err
            );

            response.set(StatusCode::BadRequest);
            return err.to_string();
        }
    }

    // pass key to authenticator
    let authenticated = authenticator(parameters.token.clone());

    if !authenticated {
        // Not authenticated, get unathorized
        response.set(StatusCode::Unauthorized);

        // Log
        warn!("{}: Rejected, Invalid Token", request.origin.remote_addr);

        return "Invalid Token".to_string();
    }

    // Get both players
    // Connect to db
    let dbcon = db::DbConnection::new();

    let player_a: &Player;
    let player_b: &Player;

    // Try to get players
    // A
    let temp_a = dbcon.get_player_by_name(&parameters.player_a);

    match &temp_a {
        Ok(player) => player_a = player,

        Err(rusqlite::Error::QueryReturnedNoRows) => {
            response.set(StatusCode::BadRequest);

            warn!(
                "{}: No player {} found (player_a)",
                request.origin.remote_addr, &parameters.player_a
            );

            return format!("{} not found", parameters.clone().player_a);
        }

        // Other misc error happened
        Err(err) => {
            response.set(StatusCode::InternalServerError);

            error!(
                "Misc error {} occured", err
            );

            return format!("Misc error {} occured", err);
        }
    }

    // B
    let temp_b = dbcon.get_player_by_name(&parameters.player_b);

    match &temp_b {
        Ok(player) => player_b = player,

        // No errors, set custom statuscode
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            response.set(StatusCode::BadRequest);

            warn!(
                "{}: No player {} found (player_b)",
                request.origin.remote_addr, &parameters.player_b
            );

            return format!("{} not found", parameters.clone().player_b);
        }

        // Other misc error happened
        Err(err) => {
            response.set(StatusCode::InternalServerError);

            error!(
                "Misc error {} occured", err
            );

            return format!("Misc error {} occured", err);
        }
    }

    // Process the game
    let rmatch = process_game(parameters.clone(), player_a, player_b);

    response.set(StatusCode::Created);
    response.set(nickel::MediaType::Json);

    // Log
    debug!("{}: Successfully created match", request.origin.remote_addr);

    // Log
    debug!("{}: Finished request", request.origin.remote_addr);

    return serde_json::to_string(&rmatch).unwrap()
}

// /api/matches/dummy
pub fn test_match(request: &mut Request, response: &mut Response) -> String {    
    // Log debug
    debug!(
        "POST /api/matches/dummy from {}",
        request.origin.remote_addr
    );

    // Convert to json
    // See if its valid
    let parameters: DummyGameStruct;

    // Lets see if everything is good
    match request.json_as() {
        Ok(params) => {
            // We parsed it properly, hooray!
            parameters = params;
        },
        Err(err) => {
            // Ya fucked up boy
            response.set(StatusCode::BadRequest);
            return err.to_string();
        }
    }

    // Get both players
    // Connect to db
    let dbcon = db::DbConnection::new();

    let player_a: &Player;
    let player_b: &Player;

    // Try to get players
    // A
    let temp_a = dbcon.get_player_by_name(&parameters.player_a);

    match &temp_a {
        Ok(player) => player_a = player,

        Err(rusqlite::Error::QueryReturnedNoRows) => {
            response.set(StatusCode::BadRequest);

            warn!(
                "{}: No player {} found (player_a)",
                request.origin.remote_addr, &parameters.player_a
            );

            return format!("{} not found", parameters.clone().player_a);
        }

        // Other misc error happened
        Err(err) => {
            response.set(StatusCode::InternalServerError);

            error!(
                "Misc error {} occured", err
            );

            return format!("Misc error {} occured", err);
        }
    }

    // B
    let temp_b = dbcon.get_player_by_name(&parameters.player_b);

    match &temp_b {
        Ok(player) => player_b = player,

        // No errors, set custom statuscode
        Err(rusqlite::Error::QueryReturnedNoRows) => {
            response.set(StatusCode::BadRequest);

            warn!(
                "{}: No player {} found (player_b)",
                request.origin.remote_addr, &parameters.player_b
            );

            return format!("{} not found", parameters.clone().player_b);
        }

        // Other misc error happened
        Err(err) => {
            response.set(StatusCode::InternalServerError);

            error!(
                "Misc error {} occured", err
            );

            return format!("Misc error {} occured", err);
        }
    }

    // If we're still "valid"
    // Process the game
    let dummymatch = process_dummy_game(parameters.clone(), player_a, player_b);

    response.set(nickel::MediaType::Json);

    // Log
    debug!(
        "{}: Successfully processed DUMMY match",
        request.origin.remote_addr
    );

    // Log
    debug!("{}: Finished request", request.origin.remote_addr);

    return serde_json::to_string(&dummymatch).unwrap();
}

// -----------------------
// PATCH REQUESTS
// will be made almost never
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
5000
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

// -----------------------
// Helping functions that used to be in main.rs
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
// Processess a game and returns a match object
fn process_game(data: GameStruct, player_a: &Player, player_b: &Player) -> Match {
   
    // Connect to db
    let dbcon = db::DbConnection::new();

    // Get the players' rank
    let player_a_rank = player_a.rank;
    let player_b_rank = player_b.rank;

    // Calc

    // Also, log info a bit more
    info!("Calculating match:");
    info!("a: {}, {} goals {} ms ({})", player_a.name, data.score_a, data.ping_a, player_a.rank);
    info!("b: {}, {} goals {} ms ({})", player_b.name, data.score_b, data.ping_b, player_b.rank);

    let new_ranks = calculations::calculate_new_rankings(
        &player_a_rank,
        &player_b_rank,
        &data.ping_a,
        &data.ping_b,
        &data.score_a,
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
    let delta_a: i16 = new_ranks.0 as i16 - player_a_rank as i16;
    let delta_b: i16 = new_ranks.1 as i16 - player_b_rank as i16;

    // Add game to db
    dbcon.add_match(
        &player_a.id.try_into().unwrap(),
        &player_b.id.try_into().unwrap(),
        &data.score_a,
        &data.score_b,
        &data.ping_a,
        &data.ping_b,
        &delta_a,
        &delta_b,
    );

    // Get the id by using lastrowid of coursor
    let rid = dbcon.conn.last_insert_rowid();

    // Also log the match id
    info!("Processed game {}.", rid);

    // Get the match by its id
    let rmatch = dbcon.get_match_by_id(&rid.try_into().unwrap()).unwrap();

    dbcon.conn.close().unwrap();

    return rmatch;
}

// Calculates scores of a game, return a dummy match object, DOESNT CHANGE ROWS
// TODO Merge process_game and process_dummy_game
fn process_dummy_game(data: DummyGameStruct, player_a: &Player, player_b: &Player) -> DetailedMatch {
    
    // Get the players' rank
    let player_a_rank = player_a.rank;
    let player_b_rank = player_b.rank;

    // Calc
    
    // Also, log info a bit more
    info!("Calculating dummy match:");
    info!("a: {}, {} goals {} ms ({})", player_a.name, data.score_a, data.ping_a, player_a.rank);
    info!("b: {}, {} goals {} ms ({})", player_b.name, data.score_b, data.ping_b, player_b.rank);

    let new_ranks = calculations::calculate_new_rankings(
        &player_a_rank,
        &player_b_rank,
        &data.ping_a,
        &data.ping_b,
        &data.score_a,
        &data.score_b,
    );

    // We usually set players ranks here, but we wont

    // Calculate rank diffference
    let delta_a: i16 = new_ranks.0 as i16 - player_a_rank as i16;
    let delta_b: i16 = new_ranks.1 as i16 - player_b_rank as i16;

    // We usually add the game to the database here, but we wont

    // Construct a Match object
    let return_match = DetailedMatch::new_dummy(
        player_a.id,
        player_b.id,
        data.score_a,
        data.score_b,
        data.ping_a,
        data.ping_b,
        delta_a,
        delta_b,
        new_ranks.2 // Here also add the debuginfo we got from calculations
    );

    return return_match;
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
        &player_b_rank,
        &data.ping_a,
        &data.ping_b,
        &data.score_a,
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
    let delta_a: i16 = player_a_rank as i16 - new_ranks.0 as i16;
    let delta_b: i16 = player_b_rank as i16 - new_ranks.1 as i16;

    // Add game to db
    dbcon.add_match(
        &player_a.id.try_into().unwrap(),
        &player_b.id.try_into().unwrap(),
        &data.score_a,
        &data.score_b,
        &data.ping_a,
        &data.ping_b,
        &delta_a,
        &delta_b,
    );

    dbcon.conn.close().unwrap();
}

// -----------------------
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
