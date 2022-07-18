// Database connection file
// Obsly uses sqlite / rusqlite
//----------------------------------------------------------------


// Imports 
// -----------------------
use rusqlite::{Connection, Result};
use serde::{Serialize, Deserialize};
use dotenv::dotenv;
use std::time::{SystemTime};
// -----------------------

// Player struct. Same as in players table
#[derive(Debug, Serialize, Deserialize)]
pub struct Player {
    pub id: usize,
    pub name: String,
    pub elo: u16,
}

// Match struct. Same as in matches table
#[derive(Debug, Serialize, Deserialize)]
pub struct Match {
    pub id: usize,
    pub player_1 : u32, // u32 as it's the player's id
    pub player_2 : u32, // Same here
    pub player_1_score : u32, // Score; 0 - 22
    pub player_2_score : u32, // Same here
    pub player_1_elo_change : i32, // Signed because it's negative for one player
    pub player_2_elo_change : i32,
    pub epoch : usize // Biggest value we can get
}

// Struct to have custom funcs based on connection
pub struct DbConnection {
    pub conn: Connection
}

impl DbConnection {

    // Make a new connection..
    pub fn new() -> Self {
        // Load env 
        dotenv().ok();

        // Get the db file from the environment
        let dbfile = std::env::var("DATABASE").unwrap();
        
        // Return connected DbConnection
        Self {conn: Connection::open(dbfile).expect("Can't connect, L")} 
    }

    // Sets up the database
    // Called on runtime
    // Doesn't do anything bad becase of IF NOT EXISTS
    pub fn setup(&mut self) -> () {

        // Create table players
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS players (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, elo INTEGER NOT NULL);",
            (), // empty list of parameters.
        ).unwrap();

        //Create table matches
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS matches (id INTEGER PRIMARY KEY AUTOINCREMENT, player_1 INTEGER NOT NULL, player_2 INTEGER NOT NULL, player_1_score INTEGER NOT NULL, player_2_score INTEGER NOT NULL, player_1_elo_change INTEGER NOT NULL, player_2_elo_change INTEGER NOT NULL, epoch INTEGER NOT NULL);",
            (), // empty list of parameters.
        ).unwrap();
    }

    // Gets a player by their name from the database
    pub fn get_player_by_name(&self, name: &str) -> Result<Player, rusqlite::Error> {
        let mut return_player = Player {
            id: 4294967295,
            name: "None".to_string(),
            elo: 0,
        };
        
        // Perform a query and match whether or not it errored
        match self.conn.query_row(
            "SELECT id, name, elo FROM players WHERE name = ?1;", &[name],
            |row| TryInto::<(usize, String, u16)>::try_into(row),
        ) {
            Ok(row) => {

                // Slap the values back in
                return_player.id = row.0;
                return_player.name = row.1;
                return_player.elo = row.2;

                Ok(return_player)
            },
            Err(err) => return Err(err),
        }
    }
    
    // Get a player by id
    pub fn get_player_by_id(&self, id: &usize) -> Result<Player, rusqlite::Error> {
        let mut return_player = Player {
            id: 4294967295,
            name: "None".to_string(),
            elo: 0,
        };

        // Perform a query and match whether or not it errored
        match self.conn.query_row(
            "SELECT id, name, elo FROM players WHERE id = ?1;", &[id],
            |row| TryInto::<(usize, String, u16)>::try_into(row),
        ) {
            Ok(row) => {

                // Slap the values back in
                return_player.id = row.0;
                return_player.name = row.1;
                return_player.elo = row.2;

                Ok(return_player)
            },
            Err(err) => return Err(err),
        }


    }

    // Get a match by id
    pub fn get_match_by_id(&self, id: &usize) -> Result<Match, rusqlite::Error> {
        // TODO: There is probably a better way to get a Match from the database
        // Like directly or by first converting to json

        let mut return_match = Match {
            id: 4294967295,
            player_1 : 0,
            player_2 : 0,
            player_1_score : 0,
            player_2_score : 0,
            player_1_elo_change : 0,
            player_2_elo_change : 0,
            epoch : 0,
        };

        // Perform a query and match whether or not it errored
        match self.conn.query_row(
            "SELECT * FROM matches WHERE id = ?1;", &[id],
            |row| TryInto::<(usize, u32, u32, u32, u32, i32, i32, usize)>::try_into(row),
        ) {
            Ok(row) => {

                // Slap the values back in
                return_match.id = row.0;
                return_match.player_1 = row.1;
                return_match.player_2 = row.2;
                return_match.player_1_score = row.3;
                return_match.player_2_score = row.4;
                return_match.player_1_elo_change = row.5;
                return_match.player_2_elo_change = row.6;
                return_match.epoch = row.7;
            
                Ok(return_match)
            },
            Err(err) => return Err(err),
        }        
    

        }

    // Set a player's name
    pub fn set_player_name_by_id(&self, id: &usize, new_name: &&str) -> Result<(), rusqlite::Error> {

        // Perform a query and match whether or not it errored
        match self.conn.execute(
            "UPDATE players SET name = ?1 WHERE id = ?2;", &[new_name, id.to_string().as_str()],
        ) {
            Ok(_) => (Ok(())),
            Err(err) => (Err(err)),
        }

    }

    // Set a player's elo
    pub fn set_player_elo_by_id(&self, id: &usize, elo: &u16) -> Result<(), rusqlite::Error> {

        // Perform a query and match whether or not it errored
        match self.conn.execute(
            "UPDATE players SET elo = ?1 WHERE id = ?2;", &[elo.to_string().as_str(), id.to_string().as_str()],
        ) {
            Ok(_) => (Ok(())),
            Err(err) => (Err(err)),
        }

    }

    // Add a row
    pub fn add_player(&self, name: &&str, elo: &u16) {

        // Do rusqlite magic!
        self.conn.execute(
            "INSERT INTO players (name, elo) VALUES (?1, ?2);", &[name.to_string().as_str(), elo.to_string().as_str(), ],
        ).unwrap();

    }

    // Add a match
    pub fn add_match(&self, player_1: &u32, player_2: &u32, &player_1_score: &u16, player_2_score: &u16, player_1_elo_change: &i16, player_2_elo_change: &i16,) {


        // Do a pain of a line
        self.conn.execute(
            "INSERT INTO matches (player_1, player_2, player_1_score, player_2_score, player_1_elo_change, player_2_elo_change, epoch) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7);", &[&player_1.to_string().as_str(), &player_2.to_string().as_str(), &player_1_score.to_string().as_str(), &player_2_score.to_string().as_str(), &player_1_elo_change.to_string().as_str(), player_2_elo_change.to_string().as_str(), &SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis().to_string()],
        ).unwrap();
    
    }

}    