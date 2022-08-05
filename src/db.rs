// Database connection file
// Obsly uses sqlite / rusqlite
//----------------------------------------------------------------

// Imports
// -----------------------
use dotenv::dotenv;
use log::warn;
use rusqlite::{params_from_iter, Connection, Result};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
// -----------------------

// Player struct. Same as in players table
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Player {
    pub id: usize,
    pub name: String,
    pub rank: u16,
}

// Match struct. Same as in matches table
#[derive(Debug, Serialize, Deserialize)]
pub struct Match {
    pub id: usize,
    pub player_a: u32, // u32 as it's the player's id
    pub player_b: u32, // Same here
    pub a_score: u32,  // Score; 0 - 22
    pub b_score: u32,  // Same here
    pub a_delta: i32,  // Signed because it's negative for one player
    pub b_delta: i32,
    pub epoch: usize, // Biggest value we can get
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
                a_score INTEGER NOT NULL,
                b_score INTEGER NOT NULL,
                a_delta INTEGER NOT NULL,
                b_delta INTEGER NOT NULL,
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
                a_score: row.get(3)?,
                b_score: row.get(4)?,
                a_delta: row.get(5)?,
                b_delta: row.get(6)?,
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
            "SELECT id, name, rank FROM players WHERE name = ?1;",
            [sanitise(name)],
            |row| TryInto::<(usize, String, u16)>::try_into(row),
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
            |row| TryInto::<(usize, String, u16)>::try_into(row),
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
            id: 4294967295,
            player_a: 0,
            player_b: 0,
            a_score: 0,
            b_score: 0,
            a_delta: 0,
            b_delta: 0,
            epoch: 0,
        };

        // Perform a query and match whether or not it errored
        match self
            .conn
            .query_row("SELECT * FROM matches WHERE id = ?1;", &[id], |row| {
                TryInto::<(usize, u32, u32, u32, u32, i32, i32, usize)>::try_into(row)
            }) {
            Ok(row) => {
                // Slap the values back in
                return_match.id = row.0;
                return_match.player_a = row.1;
                return_match.player_b = row.2;
                return_match.a_score = row.3;
                return_match.b_score = row.4;
                return_match.a_delta = row.5;
                return_match.b_delta = row.6;
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
    pub fn set_player_rank_by_id(&self, id: &usize, rank: &u16) -> Result<(), rusqlite::Error> {
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
        player_a: &u32,
        player_b: &u32,
        &a_score: &u16,
        b_score: &u16,
        a_delta: &i16,
        b_delta: &i16,
    ) {
        // Do a pain of a line
        self.conn
            .execute(
                "INSERT INTO matches (
                player_a,
                player_b,
                a_score,
                b_score,
                a_delta,
                b_delta,
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
                    &a_score.to_string().as_str(),  // ?3
                    &b_score.to_string().as_str(),  // ?4
                    &a_delta.to_string().as_str(),  // ?5
                    b_delta.to_string().as_str(),   // ?6
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

    let mut output = istr.to_lowercase().to_string();

    /*for banned_str in banned {
        if output.contains(banned_str) {
            output = output.replace(banned_str, "");
            warn!("Banned string {} found in database input, removing", banned_str);
        }
    }*/

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
