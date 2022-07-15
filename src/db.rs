// Database connection file
// Obsly uses sqlite / rusqlite
//----------------------------------------------------------------


// Imports 
// -----------------------
use rusqlite::{Connection, Result, named_params};
use serde::{Serialize, Deserialize};
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
}

// Struct to have custom funcs based on connection
pub struct DbConnection {
    pub conn: Connection
}

impl DbConnection {
    // Make a new connection..
    pub fn new() -> Self { Self {conn: Connection::open("/home/koza/code/lunars/Lunars-1/supersecretdb.sqlite").expect("Can't connect, L")} }

    // Sets up the database
    // !You should only call this once for creating a database!
    pub fn setup(&mut self) -> () {

        // Create table players
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS players (id INTEGER PRIMARY KEY AUTOINCREMENT, name TEXT NOT NULL, elo INTEGER NOT NULL);",
            (), // empty list of parameters.
        ).unwrap();

        //Create table matches
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS matches (id INTEGER PRIMARY KEY AUTOINCREMENT, player_1 INTEGER NOT NULL, player_2 INTEGER NOT NULL, player_1_score INTEGER NOT NULL, player_2_score INTEGER NOT NULL);",
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


        // Do rusqlite magic!
        let tuple: (usize, String, u16) = self.conn.query_row(
            "SELECT id, name, elo FROM players WHERE name = ?1;", &[name],
            |row| row.try_into(),
        ).unwrap();

        // Slap it back in
        return_player.id = tuple.0;
        return_player.name = tuple.1;
        return_player.elo = tuple.2;

        // This took an hour and a half to program wtf
        // Please help me

        // But like hey, at least it works?

        Ok(return_player)
    }
    
    // Get a player by id
    pub fn get_player_by_id(&self, id: &usize) -> Result<Player, rusqlite::Error> {
        let mut return_player = Player {
            id: 4294967295,
            name: "None".to_string(),
            elo: 0,
        };

        // TODO: Make it not die when no found
        // Do rusqlite magic!
        let tuple: (usize, String, u16) = self.conn.query_row(
            "SELECT id, name, elo FROM players WHERE id = ?1;", &[id],
            |row| row.try_into(),
        ).unwrap();

        // Slap it back in
        return_player.id = tuple.0;
        return_player.name = tuple.1;
        return_player.elo = tuple.2;

        Ok(return_player)
    }

    // Get a match by id
    pub fn get_match_by_id(&self, id: &usize) -> Result<Match, rusqlite::Error> {
        let mut return_match = Match {
            id: 4294967295,
            player_1 : 0,
            player_2 : 0,
            player_1_score : 0,
            player_2_score : 0,
        };
    
    
        // Do rusqlite magic!
        let tuple: (usize, u32, u32, u32, u32) = self.conn.query_row(
            "SELECT * FROM matches WHERE id = ?1;", &[id],
            |row| row.try_into(),
        ).unwrap();
    
        // Slap it back in
        return_match.id = tuple.0;
        return_match.player_1 = tuple.1;
        return_match.player_2 = tuple.2;
        return_match.player_1_score = tuple.3;
        return_match.player_2_score = tuple.4;
    
        Ok(return_match)
        }

    // Set a player's name
    pub fn set_player_name_by_id(&self, id: usize, new_name: &str) {

        // Do rusqlite magic!
        self.conn.execute(
            "UPDATE players SET name = ?1 WHERE id = ?2;", &[new_name, id.to_string().as_str()],
        ).unwrap();

    }

    // Set a player's elo
    pub fn set_player_elo_by_id(&self, id: usize, elo: u16) {

        // Do rusqlite magic!
        self.conn.execute(
            "UPDATE players SET elo = ?1 WHERE id = ?2;", &[elo.to_string().as_str(), id.to_string().as_str()],
        ).unwrap();

    }

    // Add a row
    pub fn add_player(&self, name: &str, elo: u16) {

        // Do rusqlite magic!
        self.conn.execute(
            "INSERT INTO players (name, elo) VALUES (?1, ?2);", &[name.to_string().as_str(), elo.to_string().as_str(), ],
        ).unwrap();

    }

    // Add a match
    pub fn add_match(&self, player_1: u32, player_2: u32, player_1_score: u16, player_2_score: u16,) {

        // Do rusqlite magic!
        self.conn.execute(
            "INSERT INTO matches (player_1, player_2, player_1_score, player_2_score) VALUES (?1, ?2, ?3, ?4);", &[&player_1.to_string().as_str(), &player_2.to_string().as_str(), &player_1_score.to_string().as_str(), &player_2_score.to_string().as_str(), ],
        ).unwrap();
    
    }

}    