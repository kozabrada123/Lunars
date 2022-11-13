// Database connection file
// Obsly uses sqlite / rusqlite
//----------------------------------------------------------------

// Imports
// -----------------------
use chrono;
use dotenv::dotenv;
use log::{debug, error, warn};
use nickel::{QueryString, Request};
use regex::Regex;
use rusqlite::{params_from_iter, Connection, Result};
use serde::{Deserialize, Serialize};
use std::{env, fmt::Debug, fs, time::SystemTime};
// -----------------------

// Player struct. Same as in players table
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Player {
    pub id: u64,
    pub name: String,
    pub rank: u16,       // The player's rank in the system
    pub deviation: u16,  // The player's deviation in the system
    pub volatility: f64, // The player's volatility in the system
}

// Match struct. Same as in matches table
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Match {
    pub id: u64,

    pub player_a: u64, // u64 as it's the player's id
    pub player_b: u64, // Same here

    pub score_a: u8, // Score; 0 - 22
    pub score_b: u8, // Same here

    pub ping_a: u16, // Player a's ping
    pub ping_b: u16, // Player b's

    // Glicko exclusive stuff
    pub rank_a: u16, // Players' rank sat the time
    pub rank_b: u16,

    pub deviation_a: u16, // Players' deviations at the time
    pub deviation_b: u16,

    pub volatility_a: f64, // Players' volatilities at the time
    pub volatility_b: f64,

    pub epoch: usize, // Biggest value we can get
}

impl Match {
    /*pub fn new_dummy(player_a: u64, player_b:  u64, score_a: u8, score_b: u8, delta_a: i16, delta_b: i16) -> Match {
        Match {
            id: 0, // Always just do 0, its not a valid match
            player_a: player_a,
            player_b: player_b,
            score_a: score_a,
            score_b: score_b,
            delta_a: delta_a,
            delta_b: delta_b,

            epoch:
            SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis().try_into().unwrap() }
    }*/
    // Not needed anymore, we now have DetailedMatch.

    // Sorts a match result by a player id.
    // basically makes the that player always player a
    pub fn sort_by_player_id(&self, player_id: u64) -> Match {
        let mut sorted = self.clone();

        // Check if it even needs to be sorted
        // If the desired player is already we don't need to sort it at all
        if sorted.player_a == player_id {
            return sorted;
        }

        // If it does need to be sorted, sort it

        // b = a
        // a = b

        sorted.player_a = self.player_b;
        sorted.player_b = self.player_a;

        sorted.ping_a = self.ping_b;
        sorted.ping_b = self.ping_a;

        sorted.score_a = self.score_b;
        sorted.score_b = self.score_a;

        sorted.rank_a = self.rank_b;
        sorted.rank_b = self.rank_a;

        sorted.deviation_a = self.deviation_b;
        sorted.deviation_b = self.deviation_a;

        sorted.volatility_a = self.volatility_b;
        sorted.volatility_b = self.volatility_a;

        return sorted;
    }
}

// Match struct with extra calculation details. Only to be used for dummy matches and testing
#[derive(Debug, Serialize, Deserialize)]
pub struct DetailedMatch {
    pub id: u64,
    pub player_a: u64, // u64 as it's the player's id
    pub player_b: u64, // Same

    pub score_a: u8, // Score; 0 - 22
    pub score_b: u8, // Same here

    pub ping_a: u16, // Player a's ping
    pub ping_b: u16, // Player b's

    // Glicko exclusive stuff
    pub rank_a: u16, // Players' rank sat the time
    pub rank_b: u16,

    pub deviation_a: u16, // Players' deviations at the time
    pub deviation_b: u16,

    pub volatility_a: f64, // Players' volatilities at the time
    pub volatility_b: f64,

    pub epoch: usize, // Biggest value we can get

    pub debuginfo: DebugInfo,
}

impl DetailedMatch {
    pub fn new_dummy(
        player_a: u64,
        player_b: u64,
        score_a: u8,
        score_b: u8,
        ping_a: u16,
        ping_b: u16,
        rank_a: u16,
        rank_b: u16,
        deviation_a: u16,
        deviation_b: u16,
        volatility_a: f64,
        volatility_b: f64,
        debuginfo: DebugInfo,
    ) -> DetailedMatch {
        DetailedMatch {
            id: 0, // Always just do 0, its not a valid match
            player_a: player_a,
            player_b: player_b,

            score_a: score_a,
            score_b: score_b,

            ping_a: ping_a,
            ping_b: ping_b,

            rank_a: rank_a,
            rank_b: rank_b,

            deviation_a: deviation_a,
            deviation_b: deviation_b,

            volatility_a: volatility_a,
            volatility_b: volatility_b,

            debuginfo: debuginfo, // Also include the debug info

            epoch: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis()
                .try_into()
                .unwrap(),
        }
    }
}

// Debug info for DetailedMatch so we can have strongly typed datatypes
#[derive(Debug, Serialize, Deserialize)]
pub struct DebugInfo {
    pub time: u64,      //time it took to process calculations, in μs
    pub ability_a: u64, // player's ability, expressed in a u64. Abilitys can sometimes be floats, but we can discard the .08 left over as it doesnt matter
    pub ability_b: u64,
    pub expected_a: f32, // expected score distribution, between 0 and 1
    pub expected_b: f32,
    pub actual_a: f32, // actual score distribution, same as expected; between 0 and 1
    pub actual_b: f32,
}

// Struct to have custom funcs based on connection
pub struct DbConnection {
    pub conn: Connection,
}

impl DbConnection {
    // Make a new connection..
    pub fn new() -> Self {
        // Load env
        dotenv().ok();

        // Get the db file from the environment
        let dbfile = std::env::var("DATABASE").unwrap();

        // Return connected DbConnection
        Self {
            conn: Connection::open(dbfile).expect("Can't connect, L"),
        }
    }

    // Make a new test connection with a supplied name
    // Only used for tests
    #[allow(dead_code)]
    pub fn new_named(name: &str) -> Self {
        // Return connected DbConnection
        Self {
            conn: Connection::open(&name).expect("Can't connect, L"),
        }
    }

    // Sets up the database
    // Called on runtime
    // Doesn't do anything bad becase of IF NOT EXISTS
    pub fn setup(&mut self) -> () {
        // Create table players
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS players (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                rank INTEGER NOT NULL,
                deviation INTEGER NOT NULL,
                volatility INTEGER NOT NULL
            );",
                (), // empty list of parameters.
            )
            .unwrap();

        //Create table matches
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS matches (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                player_a INTEGER NOT NULL,
                player_b INTEGER NOT NULL,

                score_a INTEGER NOT NULL,
                score_b INTEGER NOT NULL,

                ping_a INTEGER NOT NULL,
                ping_b INTEGER NOT NULL,

                delta_a INTEGER NOT NULL,
                delta_b INTEGER NOT NULL,
                
                rating_a INTEGER NOT NULL,
                rating_b INTEGER NOT NULL,
                
                deviation_a INTEGER NOT NULL,
                deviation_b INTEGER NOT NULL,
                
                volatility_a INTEGER NOT NULL,
                volatility_b INTEGER NOT NULL,

                epoch INTEGER NOT NULL
            );",
                (), // empty list of parameters.
            )
            .unwrap();
    }

    // Parses get players sql
    // Rewritten 31-07-22 so that we can have sort by among other things
    // Default is "SELECT * FROM players" fyi
    // First arg is sql to get the players, second is sql parameters
    // i.e "SELECT * FROM players where id = ?1" &[player.id.to_string()]
    pub fn get_players(&self, sql: &str, param: &[String]) -> Result<Vec<Player>, rusqlite::Error> {
        // Get players
        let mut query = self.conn.prepare(sql)?;
        let player_iter = query.query_map(params_from_iter(param), |row| {
            Ok(Player {
                id: row.get(0)?,
                name: row.get(1)?,
                rank: row.get(2)?,
                deviation: row.get(3)?,
                volatility: row.get(4)?,
            })
        })?;

        // Stuff them in a vector (slightly inefficiently)
        let mut players = Vec::<Player>::new();

        for player in player_iter {
            players.push(player.unwrap());
        }

        Ok(players)
    }

    // Parses get matches sql
    // Rewritten 31-07-22 so that we can have sort by among other things
    // Default is "SELECT * FROM matches" fyi
    // First arg is sql to get the matches, second is sql parameters
    // i.e "SELECT * FROM matches where id = ?1" &[match.id.to_string()]
    pub fn get_matches(&self, sql: &str, param: &[String]) -> Result<Vec<Match>, rusqlite::Error> {
        // Get matches
        let mut query = self.conn.prepare(sql)?;
        let match_iter = query.query_map(params_from_iter(param), |row| {
            Ok(Match {
                id: row.get(0)?,

                player_a: row.get(1)?,
                player_b: row.get(2)?,

                score_a: row.get(3)?,
                score_b: row.get(4)?,

                ping_a: row.get(5)?,
                ping_b: row.get(6)?,

                rank_a: row.get(7)?,
                rank_b: row.get(8)?,

                deviation_a: row.get(9)?,
                deviation_b: row.get(10)?,

                volatility_a: row.get(11)?,
                volatility_b: row.get(12)?,

                epoch: row.get(13)?,
            })
        })?;

        // Stuff them in a vector (slightly inefficiently)
        let mut matches = Vec::<Match>::new();

        for smatch in match_iter {
            matches.push(smatch.unwrap());
        }

        Ok(matches)
    }

    // Gets a player by their name from the database
    pub fn get_player_by_name(&self, name: &str) -> Result<Player, rusqlite::Error> {
        let mut return_player = Player {
            id: 0,
            name: "None".to_string(),
            rank: 0,
            deviation: 0,
            volatility: 0.0,
        };

        // Perform a query and match whether or not it errored
        match self.conn.query_row(
            "SELECT * FROM players WHERE name = ?1 COLLATE NOCASE;",
            [sanitise(name)],
            |row| TryInto::<(u64, String, u16, u16, f64)>::try_into(row),
        ) {
            Ok(row) => {
                // Slap the values back in
                return_player.id = row.0;
                return_player.name = row.1;
                return_player.rank = row.2;
                return_player.deviation = row.3;
                return_player.volatility = row.4;

                Ok(return_player)
            }
            Err(err) => return Err(err),
        }
    }

    // Get a player by id
    pub fn get_player_by_id(&self, id: &usize) -> Result<Player, rusqlite::Error> {
        let mut return_player = Player {
            id: 0,
            name: "None".to_string(),
            rank: 0,
            deviation: 0,
            volatility: 0.0,
        };

        // Perform a query and match whether or not it errored
        match self.conn.query_row(
            "SELECT id, name, rank FROM players WHERE id = ?1;",
            &[id],
            |row| TryInto::<(u64, String, u16, u16, f64)>::try_into(row),
        ) {
            Ok(row) => {
                // Slap the values back in
                return_player.id = row.0;
                return_player.name = row.1;
                return_player.rank = row.2;
                return_player.deviation = row.3;
                return_player.volatility = row.4;

                Ok(return_player)
            }
            Err(err) => return Err(err),
        }
    }

    // Get a match by id
    pub fn get_match_by_id(&self, id: &usize) -> Result<Match, rusqlite::Error> {
        // TODO: There is probably a better way to get a Match from the database
        // Like directly or by first converting to json

        let mut return_match = Match {
            id: 0,

            player_a: 0,
            player_b: 0,

            score_a: 0,
            score_b: 0,

            ping_a: 0,
            ping_b: 0,

            rank_a: 0,
            rank_b: 0,

            deviation_a: 0,
            deviation_b: 0,

            volatility_a: 0.0,
            volatility_b: 0.0,

            epoch: 0,
        };

        // Perform a query and match whether or not it errored
        match self
            .conn
            .query_row("SELECT * FROM matches WHERE id = ?1;", &[id], |row| {
                TryInto::<(
                    u64,
                    u64,
                    u64,
                    u8,
                    u8,
                    u16,
                    u16,
                    u16,
                    u16,
                    u16,
                    u16,
                    f64,
                    f64,
                    usize,
                )>::try_into(row)
            }) {
            Ok(row) => {
                // Slap the values back in
                return_match.id = row.0;

                return_match.player_a = row.1;
                return_match.player_b = row.2;

                return_match.score_a = row.3;
                return_match.score_b = row.4;

                return_match.ping_a = row.5;
                return_match.ping_b = row.6;

                return_match.rank_a = row.7;
                return_match.rank_b = row.8;

                return_match.deviation_a = row.9;
                return_match.deviation_b = row.10;

                return_match.volatility_a = row.11;
                return_match.volatility_b = row.12;

                return_match.epoch = row.13;

                Ok(return_match)
            }
            Err(err) => return Err(err),
        }
    }

    // Set a player's name
    // Never used lmao
    /*
    pub fn set_player_name_by_id(&self, id: &usize, new_name: &&str) -> Result<(), rusqlite::Error> {

        // Perform a query and match whether or not it errored
        match self.conn.execute(
            "UPDATE players SET name = ?1 WHERE id = ?2;", &[sanitise(new_name).as_str(), id.to_string().as_str()],
        ) {
            Ok(_) => (Ok(())),
            Err(err) => (Err(err)),
        }

    }*/

    // Set a player's rank
    pub fn set_player_rank_by_id(&self, id: &u64, rank: &u16) -> Result<(), rusqlite::Error> {
        // Perform a query and match whether or not it errored
        match self.conn.execute(
            "UPDATE players SET rank = ?1 WHERE id = ?2;",
            &[rank.to_string().as_str(), id.to_string().as_str()],
        ) {
            Ok(_) => (Ok(())),
            Err(err) => (Err(err)),
        }
    }

    // Add a row
    pub fn add_player(&self, name: &&str, rank: &u16) {
        // Do rusqlite magic!
        self.conn
            .execute(
                "INSERT INTO players (name, rank) VALUES (?1, ?2);",
                &[sanitise(name).as_str(), rank.to_string().as_str()],
            )
            .unwrap();
    }

    // Add a match
    pub fn add_match(
        &self,

        player_a: &u64,
        player_b: &u64,

        score_a: &u8,
        score_b: &u8,

        ping_a: &u16,
        ping_b: &u16,

        delta_a: &i16,
        delta_b: &i16,

        rank_a: &u16,
        rank_b: &u16,

        deviation_a: &u16,
        deviation_b: &u16,

        volatility_a: &f64,
        volatility_b: &f64,
    ) {
        // Do a pain of a line
        self.conn
            .execute(
                "INSERT INTO matches (
                player_a,
                player_b,
                score_a,
                score_b,
                ping_a,
                ping_b,
                delta_a,
                delta_b,
                rank_a,
                rank_b,
                deviation_a,
                deviation_b,
                volatility_a,
                volatility_b,
                epoch
            ) VALUES (
                ?1,
                ?2,
                ?3,
                ?4,
                ?5,
                ?6,
                ?7,
                ?8,
                ?9
                ?10,
                ?11,
                ?12,
                ?13,
                ?14,
                ?15
            );",
                &[
                    player_a.to_string().as_str(),     // ?1
                    player_b.to_string().as_str(),     // ?2
                    score_a.to_string().as_str(),      // ?3
                    score_b.to_string().as_str(),      // ?
                    ping_a.to_string().as_str(),       // ?5
                    ping_b.to_string().as_str(),       // ?6
                    delta_a.to_string().as_str(),      // ?7
                    delta_b.to_string().as_str(),      // ?8
                    rank_a.to_string().as_str(),       // ?9
                    rank_b.to_string().as_str(),       // ?10
                    deviation_a.to_string().as_str(),  // ?11
                    deviation_b.to_string().as_str(),  // ?12
                    volatility_a.to_string().as_str(), // ?13
                    volatility_b.to_string().as_str(), // ?14
                    SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                        .to_string()
                        .as_str(), // ?15
                ],
            )
            .unwrap();
    }
}

pub fn sanitise(istr: &str) -> String {
    /*let banned = vec![
        "add",
        "alter",
        "column",
        "row",
        "create",
        "delete",
        "drop",
        "where",
        "exec",
        "null",
        "select",
        "truncate",
        "insert",
        "drop",
        "into",
        "table",
        "tables",
        "values",
        "players",
        "player",
        "match",
        "matches",
        "database",
        r"\",
        "(",
        ")",
        "[",
        "]",
        "{",
        "}",
        "=",
        " ",
        "'",
        r#"""#, // string "
        ";",
        "&",
        "*",
        "$",
        "|",
        "?",
        "~",
        "<",
        ">",
        "^",
    ];*/

    let mut output = istr.to_string();

    /*for banned_str in banned {
        if output.contains(banned_str) {
            output = output.replace(banned_str, "");
            warn!("Banned string {} found in database input, removing", banned_str);
        }
    }*/

    if !Regex::new("[A-Za-z0-9_.-]{2,24}")
        .unwrap()
        .is_match(&output)
    {
        return output;
    }

    // Warframe's username filter
    for char in output.clone().chars() {
        // Only Letters, Numbers, Periods, Under-Scores and Hyphens
        if !(char.is_alphanumeric() || ['.', '-', '_'].contains(&char)) {
            // If it isnt any of the above, remove all occurences of the char
            output = output.replace(char, "");
            warn!(
                "Database input contains {} char (not alphanumeric), removing",
                char
            );
        }
    }

    output
}

// backups func
pub fn backup() {
    // Check if the var is valid
    dotenv().ok();
    let backupdir = env::var("BACKUPDIR").unwrap();

    let dbfile = env::var("DATABASE").unwrap();

    // See if its a valid directory
    fs::read_dir(&backupdir).unwrap();

    debug!("Performing backup..");

    // Backup
    let backupresult = fs::copy(
        &dbfile,
        format!(
            "{}/backup{}.db",
            backupdir,
            chrono::Local::today().format("%Y-%m-%d")
        ),
    );

    // See whether or not it worked
    match backupresult {
        Ok(_res) => {
            debug!(
                "Successfully performed backup into {}",
                format!(
                    "{}/backup{}.db",
                    backupdir,
                    chrono::Local::today().format("%Y-%m-%d")
                )
            );
        }
        Err(err) => {
            error!("Failed to create backup! {}", err);
        }
    }
}

// Function that takes url args (?max, ?min, ?player...) and makes an sql query
/*

Usage:
base = "SELECT * FROM players"
request is your request

based on determined parameters, this returns the modified query.

for example, if request here has a parameter ?max=x, then this will return
something like "SELECT * FROM players WHERE rank <= x"

valid params to be implemented:

/api/players:
?max=x, where x is the max rank value
?min=x, where x is the minimum rank value

/api/matches:
?after=x, where x is a unix timestamp
?before=x, where x is a unix timestamp
?has_player=x, where x is a player id or name that we want to be in the match

*/
pub fn build_query(base: String, request: &mut Request, firstused: bool) -> String {
    // First, clone the base query
    let mut query = base.clone();

    // Then lets go through the list of parameters
    // Firstly, keep in mind whether or not this is the first

    // firstused is a parameter that tells us if we've used the first parameter yet
    // so we can do stuff like WHERE player = a and then still pass that into the buildquery
    let mut firstparam = !firstused;

    // Also keep in mind the amount of order_by and sorts so that we know where we need a comma
    //let mut firstsort = true;

    /*
    We use firstparam to know whether or not we need to use WHERE or AND.
    If this is the first one, we need to first add WHERE
    If it isnt, we need to add AND
    */

    //max & min
    if request.query().get("max").is_some() {
        // Log for debugging
        debug!(
            "Got valid url parameter max: {}",
            request.query().get("max").unwrap()
        );

        // Keep track of what we want to add to the query
        let mut to_add = String::new();

        // We do have a max, see if it's purely a number (pls no sql injection
        let mut max: Option<usize> = None;

        match request.query().get("max").unwrap().parse::<usize>() {
            Ok(parsed) => {
                // It is a valid number
                max = Some(parsed);
            }
            Err(_e) => {
                // It isnt a valid number, don't set max so it'll be None
            }
        }

        if max.is_some() {
            // we're fine, continue on

            // check if we need a WHERE or an AND
            match firstparam {
                true => {
                    // We need a WHERE
                    to_add.push_str(" WHERE ");

                    // Also now that we've added the WHERE, make it so others know that we did
                    firstparam = false;
                }
                false => {
                    // We need an AND
                    to_add.push_str(" AND ");
                }
            }

            // Now actually add
            to_add.push_str(format!("rank <= {}", max.unwrap().to_string()).as_str());
            query.push_str(to_add.as_str());
        }
    }

    if request.query().get("min").is_some() {
        // Log for debugging
        debug!(
            "Got valid url parameter min: {}",
            request.query().get("min").unwrap()
        );

        // Keep track of what we want to add to the query
        let mut to_add = String::new();

        // We do have a min, see if it's purely a number (pls no sql injection
        let mut min: Option<usize> = None;

        match request.query().get("min").unwrap().parse::<usize>() {
            Ok(parsed) => {
                // It is a valid number
                min = Some(parsed);
            }
            Err(_e) => {
                // It isnt a valid number, don't set max so it'll be None
            }
        }

        if min.is_some() {
            // we're fine, continue on

            // check if we need a WHERE or an AND
            match firstparam {
                true => {
                    // We need a WHERE
                    to_add.push_str(" WHERE ");

                    // Also now that we've added the WHERE, make it so others know that we did
                    firstparam = false;
                }
                false => {
                    // We need an AND
                    to_add.push_str(" AND ");
                }
            }

            // Now actually add
            to_add.push_str(format!("rank >= {}", min.unwrap().to_string()).as_str());
            query.push_str(to_add.as_str());
        }
    }

    //player, only for /api/matches
    if request.query().get("has_player").is_some() {
        // Log for debugging
        debug!(
            "Got valid url parameter has_player: {}",
            request.query().get("has_player").unwrap()
        );

        // Keep track of what we want to add to the query
        let mut to_add = String::new();

        // Check if they're a real player
        // If they are we're gonna set player to their id
        let mut player: Option<u64> = None;

        // First see if its an id or not
        match request.query().get("has_player").unwrap().parse::<u64>() {
            Ok(id) => {
                // We have an id, just set that
                player = Some(id);
            }
            // Unsuccessful parse, its a string that cant be parsed into a number
            Err(_err) => {
                // We have a player's name, very annoying, lets see if its a valid one

                // Connect to db
                let dbcon = DbConnection::new();

                // Get the player
                let temp = dbcon.get_player_by_name(
                    sanitise(request.query().get("has_player").unwrap()).as_str(),
                );

                //Try Check
                match &temp {
                    Ok(res) => {
                        // Real player, yay
                        // set player to the id
                        player = Some(res.id);
                    }

                    // Not a real player aaaaaaaaaaaaaaaaaaa
                    Err(rusqlite::Error::QueryReturnedNoRows) => {
                        // Leave player alone to be none

                        warn!("{}: No player {} found", request.origin.remote_addr, &query);
                    }

                    // Other misc error happened
                    Err(err) => {
                        error!(
                            "{}: Misc error {} happened",
                            request.origin.remote_addr, err
                        );
                    }
                }
            }
        }

        if player.is_some() {
            // we're fine and have a player, continue on

            // check if we need a WHERE or an AND
            match firstparam {
                true => {
                    // We need a WHERE
                    to_add.push_str(" WHERE ");

                    // Also now that we've added the WHERE, make it so others know that we did
                    firstparam = false;
                }
                false => {
                    // We need an AND
                    to_add.push_str(" AND ");
                }
            }

            // Now actually add
            to_add.push_str(
                format!(
                    "(player_a = {} OR player_b = {})",
                    player.unwrap().to_string(),
                    player.unwrap().to_string()
                )
                .as_str(),
            );
            query.push_str(to_add.as_str());
        }
    }

    // before & after
    if request.query().get("before").is_some() {
        // Log for debugging
        debug!(
            "Got valid url parameter before: {}",
            request.query().get("before").unwrap()
        );

        // Keep track of what we want to add to the query
        let mut to_add = String::new();

        // We do have a before, see if it's purely a number (pls no sql injection
        let mut before: Option<usize> = None;

        match request.query().get("before").unwrap().parse::<usize>() {
            Ok(parsed) => {
                // It is a valid number
                before = Some(parsed);
            }
            Err(_e) => {
                // It isnt a valid number, don't set before so it'll be None
            }
        }

        if before.is_some() {
            // we're fine, continue on

            // check if we need a WHERE or an AND
            match firstparam {
                true => {
                    // We need a WHERE
                    to_add.push_str(" WHERE ");

                    // Also now that we've added the WHERE, make it so others know that we did
                    firstparam = false;
                }
                false => {
                    // We need an AND
                    to_add.push_str(" AND ");
                }
            }

            // Now actually add
            to_add.push_str(format!("epoch <= {}", before.unwrap().to_string()).as_str());
            query.push_str(to_add.as_str());
        }
    }

    if request.query().get("after").is_some() {
        // Log for debugging
        debug!(
            "Got valid url parameter after: {}",
            request.query().get("after").unwrap()
        );

        // Keep track of what we want to add to the query
        let mut to_add = String::new();

        // We do have a after, see if it's purely a number (pls no sql injection
        let mut after: Option<usize> = None;

        match request.query().get("after").unwrap().parse::<usize>() {
            Ok(parsed) => {
                // It is a valid number
                after = Some(parsed);
            }
            Err(_e) => {
                // It isnt a valid number, don't set after so it'll be None
            }
        }

        if after.is_some() {
            // we're fine, continue on

            // check if we need a WHERE or an AND
            match firstparam {
                true => {
                    // We need a WHERE
                    to_add.push_str(" WHERE ");

                    // Also now that we've added the WHERE, make it so others know that we did
                    firstparam = false;
                }
                false => {
                    // We need an AND
                    to_add.push_str(" AND ");
                }
            }

            // Now actually add
            to_add.push_str(format!("epoch >= {}", after.unwrap().to_string()).as_str());
            query.push_str(to_add.as_str());
        }
    }

    // sort / order by
    if request.query().get("sort").is_some() {
        // Log for debugging
        debug!(
            "Got valid url parameter sort: {}",
            request.query().get("sort").unwrap()
        );

        // Keep track of what we want to add to the query
        let mut to_add = String::new();

        // We do have a max, see if it's purely a number (pls no sql injection

        // We don't need to check what parameter we are because we should always be the last.
        // Nothing should come after us.

        // Parse the sort param
        // What we want is something like sort="rank|asc,name|desc"
        // , defines different sorts
        // | is the deliminator between column names and asc / desc

        // Keep the sorts in mind with a vec
        let mut sorts = Vec::<&str>::new();

        // First check if we have commas
        if request.query().get("sort").unwrap().contains(",") {
            // It does, we can split on them
            sorts = request.query().get("sort").unwrap().split(",").collect();
        } else {
            // No it doesn't, we'll only have one sort
            sorts.push(request.query().get("sort").unwrap());
        }

        // Iterate through the sorts
        // Keep in mind whether we're the first
        let mut firstsort = true;

        for s in sorts.iter() {
            // Check if it's valid
            // Keep in mind what col and what type of sort we want
            let col: String;
            let mut typ = "".to_string();

            // First check if it has a | divider
            if s.clone().contains("|") {
                // If it does the col is everything up to the divider
                // Split it then
                let split = s.split("|").collect::<Vec<&str>>(); // Horrific line
                col = sanitise(split[0].to_string().as_str());

                let temptyp = split[1].to_string().to_lowercase();

                // And we can check if the split type is valid
                match temptyp.as_str() {
                    "asc" => {
                        typ = "asc".to_string();
                    }
                    "desc" => {
                        typ = "desc".to_string();
                    }
                    &_ => {
                        // None of them, we deal with this later
                    }
                }
            } else {
                // It doesn't, we can assume that its all col
                col = sanitise(s.clone());
            }

            // Check if we have a typ
            if typ == "" {
                // We don't, use the default
                match col.as_str() {
                    "id" => {
                        typ = "asc".to_string();
                    }
                    "player_a" => {
                        typ = "asc".to_string();
                    }
                    "player_b" => {
                        typ = "asc".to_string();
                    }
                    "rank" => {
                        typ = "desc".to_string();
                    }
                    &_ => {
                        // None of the specific ones, set it to asc
                        typ = "asc".to_string();
                    }
                }
            }

            // We do or we got one, we can finally add to the sort
            // Check if we need a order_by or a comma
            match firstsort {
                true => {
                    // We need an ORDER_BY
                    to_add.push_str(" ORDER BY ");

                    // Also now that we've added the WHERE, make it so others know that we did
                    firstsort = false;
                }
                false => {
                    // We need a comma
                    to_add.push_str(", ");
                }
            }

            // finally add the order_bys
            debug!("Adding to sort {} {}", col, typ);
            to_add.push_str(format!("{} {}", col, typ).as_str());
        }

        // Now actually add
        query.push_str(to_add.as_str());
    }

    // limit & offset
    if request.query().get("limit").is_some() {
        // Log for debugging
        debug!(
            "Got valid url parameter limit: {}",
            request.query().get("limit").unwrap()
        );

        // Keep track of what we want to add to the query
        let mut to_add = String::new();

        // We do have a limit, see if it's purely a number (pls no sql injection
        let mut limit: Option<usize> = None;

        match request.query().get("limit").unwrap().parse::<usize>() {
            Ok(parsed) => {
                // It is a valid number
                limit = Some(parsed);
            }
            Err(_e) => {
                // It isnt a valid number, don't set limit so it'll be None
            }
        }

        if limit.is_some() {
            // we're fine, continue on

            // We don't need to check what parameter we are because we should always be the last.
            // Nothing should come after us.

            // Now actually add
            to_add.push_str(format!(" LIMIT {}", limit.unwrap().to_string()).as_str());
            query.push_str(to_add.as_str());
        }
    }

    if request.query().get("offset").is_some() {
        // Log for debugging
        debug!(
            "Got valid url parameter offset: {}",
            request.query().get("offset").unwrap()
        );

        // Keep track of what we want to add to the query
        let mut to_add = String::new();

        // We do have a offset, see if it's purely a number (pls no sql injection
        let mut offset: Option<usize> = None;

        match request.query().get("offset").unwrap().parse::<usize>() {
            Ok(parsed) => {
                // It is a valid number
                offset = Some(parsed);
            }
            Err(_e) => {
                // It isnt a valid number, don't set max so it'll be None
            }
        }

        if offset.is_some() {
            // we're fine, continue on

            // We don't need to check what parameter we are because we should always be the last.
            // Nothing should come after us.

            // Now actually add
            to_add.push_str(format!(" OFFSET {}", offset.unwrap().to_string()).as_str());
            query.push_str(to_add.as_str());
        }
    }

    // Lastly also add a ;
    query.push_str(";");

    // Print for debugging
    debug!("Parsed url params into query: {}", query.clone());

    // Finally, return query
    return query;
}
