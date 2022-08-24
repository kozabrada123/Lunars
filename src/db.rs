// Database connection file
// Obsly uses sqlite / rusqlite
//----------------------------------------------------------------

// Imports
// -----------------------
use dotenv::dotenv;
use log::{warn, error, debug};
use rusqlite::{params_from_iter, Connection, Result};
use serde::{Deserialize, Serialize};
use std::{time::SystemTime, fs, env};
use regex::Regex;
use chrono;
// -----------------------

// Player struct. Same as in players table
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Player {
    pub id: u64,
    pub name: String,
    pub rank: u16,
}

// Match struct. Same as in matches table
#[derive(Debug, Serialize, Deserialize)]
pub struct Match {
    pub id: u64,
    pub player_a: u64, // u64 as it's the player's id
    pub player_b: u64, // Same here
    pub score_a: u8,  // Score; 0 - 22
    pub score_b: u8,  // Same here
    pub delta_a: i16,  // Signed because it's negative for one player
    pub delta_b: i16,
    pub epoch: usize, // Biggest value we can get
}

impl Match {
    pub fn new_dummy(player_a: u64, player_b:  u64, score_a: u8, score_b: u8, delta_a: i16, delta_b: i16) -> Match {
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
    }
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
                rank INTEGER NOT NULL
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
                delta_a INTEGER NOT NULL,
                delta_b INTEGER NOT NULL,
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
                delta_a: row.get(5)?,
                delta_b: row.get(6)?,
                epoch: row.get(7)?,
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
            id: 4294967295,
            name: "None".to_string(),
            rank: 0,
        };

        // Perform a query and match whether or not it errored
        match self.conn.query_row(
            "SELECT id, name, rank FROM players WHERE name = ?1 COLLATE NOCASE;",
            [sanitise(name)],
            |row| TryInto::<(u64, String, u16)>::try_into(row),
        ) {
            Ok(row) => {
                // Slap the values back in
                return_player.id = row.0;
                return_player.name = row.1;
                return_player.rank = row.2;

                Ok(return_player)
            }
            Err(err) => return Err(err),
        }
    }

    // Get a player by id
    pub fn get_player_by_id(&self, id: &usize) -> Result<Player, rusqlite::Error> {
        let mut return_player = Player {
            id: 4294967295,
            name: "None".to_string(),
            rank: 0,
        };

        // Perform a query and match whether or not it errored
        match self.conn.query_row(
            "SELECT id, name, rank FROM players WHERE id = ?1;",
            &[id],
            |row| TryInto::<(u64, String, u16)>::try_into(row),
        ) {
            Ok(row) => {
                // Slap the values back in
                return_player.id = row.0;
                return_player.name = row.1;
                return_player.rank = row.2;

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
            delta_a: 0,
            delta_b: 0,
            epoch: 0,
        };

        // Perform a query and match whether or not it errored
        match self
            .conn
            .query_row("SELECT * FROM matches WHERE id = ?1;", &[id], |row| {
                TryInto::<(u64, u64, u64, u8, u8, i16, i16, usize)>::try_into(row)
            }) {
            Ok(row) => {
                // Slap the values back in
                return_match.id = row.0;
                return_match.player_a = row.1;
                return_match.player_b = row.2;
                return_match.score_a = row.3;
                return_match.score_b = row.4;
                return_match.delta_a = row.5;
                return_match.delta_b = row.6;
                return_match.epoch = row.7;

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
        delta_a: &i16,
        delta_b: &i16,
    ) {
        // Do a pain of a line
        self.conn
            .execute(
                "INSERT INTO matches (
                player_a,
                player_b,
                score_a,
                score_b,
                delta_a,
                delta_b,
                epoch
            ) VALUES (
                ?1,
                ?2,
                ?3,
                ?4,
                ?5,
                ?6,
                ?7
            );",
                &[
                    &player_a.to_string().as_str(), // ?1
                    &player_b.to_string().as_str(), // ?2
                    &score_a.to_string().as_str(),  // ?3
                    &score_b.to_string().as_str(),  // ?4
                    &delta_a.to_string().as_str(),  // ?5
                    delta_b.to_string().as_str(),   // ?6
                    &SystemTime::now()
                        .duration_since(SystemTime::UNIX_EPOCH)
                        .unwrap()
                        .as_millis()
                        .to_string(), // ?7
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

    if !Regex::new("[A-Za-z0-9_.-]{4,24}").unwrap().is_match(&output) {return output;}

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
    let backupresult = fs::copy(&dbfile, format!("{}/backup{}.db", backupdir, chrono::Local::today().format("%Y-%m-%d")));

    // See whether or not it worked
    match backupresult {
        Ok(_res) => {
            debug!("Successfully performed backup into {}", format!("{}/backup{}.db", backupdir, chrono::Local::today().format("%Y-%m-%d")));
        },
        Err(err) => {
            error!("Failed to create backup! {}", err);
        }
    }
}