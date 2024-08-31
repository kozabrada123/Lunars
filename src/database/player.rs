use core::panic;

use rocket::form::validate::Len;
use sqlx::mysql::MySqlQueryResult;

use crate::types::entities::player::Player;

use super::DbConnection;

impl DbConnection {
    /// Fetches all the players.
    // TODO: implement the fancy url parameters we had
    // (https://github.com/kozabrada123/Lunars/blob/885a8b4ef9ac4f08a45106fe9f5cdc397a81d72c/src/db.rs#L285)
    pub async fn get_players(&mut self) -> Vec<Player> {
        let query_string = "SELECT * FROM players";

        let result = sqlx::query_as(&query_string)
            .fetch_all(&mut **self.inner)
            .await;

        match result {
            Ok(vec) => return vec,
            Err(e) => match e {
                sqlx::Error::RowNotFound => return Vec::new(),
                _ => {
                    log::error!("Database query failed {} -> {}", query_string, e);
                    panic!("Database query failed");
                }
            },
        }
    }

    /// Fetches a player by name
    pub async fn get_player_by_name(&mut self, name: &str) -> Option<Player> {
        let query_string = "SELECT * FROM players WHERE name = ?1 COLLATE NOCASE";

        let query = sqlx::query_as(&query_string).bind(name);

        let result: Result<Vec<Player>, sqlx::Error> = query.fetch_all(&mut **self.inner).await;

        match result {
            Ok(vec) => {
                if vec.len() != 1 {
                    log::warn!("More than one player has name {}!", name);
                }

                return Some(vec[0].clone());
            }
            Err(e) => match e {
                sqlx::Error::RowNotFound => return None,
                _ => {
                    log::error!("Database query failed {} -> {}", query_string, e);
                    panic!("Database query failed");
                }
            },
        }
    }

    /// Fetches a player by id
    pub async fn get_player_by_id(&mut self, id: u64) -> Option<Player> {
        let query_string = "SELECT * FROM players WHERE id = ?1";

        let query = sqlx::query_as(&query_string).bind(id);

        let result: Result<Player, sqlx::Error> = query.fetch_one(&mut **self.inner).await;

        match result {
            Ok(player) => {
                return Some(player);
            }
            Err(e) => match e {
                sqlx::Error::RowNotFound => return None,
                _ => {
                    log::error!("Database query failed {} -> {}", query_string, e);
                    panic!("Database query failed");
                }
            },
        }
    }

    /// Updates a player.
    ///
    /// Every field can be changed except id.
    pub async fn modify_player(
        &mut self,
        player: &Player,
    ) -> Result<MySqlQueryResult, sqlx::Error> {
        let query_string = "UPDATE players SET name = ?2, rating = ?3, deviation = ?4, volatility = ?5 WHERE id = ?1";

        let query = sqlx::query(&query_string)
            .bind(player.id)
            .bind(&player.name)
            .bind(player.rating)
            .bind(player.deviation)
            .bind(player.volatility);

        let result = query.execute(&mut **self.inner).await;

        match result {
            Ok(result) => {
                return Ok(result);
            }
            Err(e) => match e {
                _ => {
                    log::error!("Database query failed {} -> {}", query_string, e);
                    panic!("Database query failed");
                }
            },
        }
    }

	 /// Adds a player.
	 ///
	 /// Ignores the id field.
    pub async fn add_player(
        &mut self,
        player: &Player,
    ) -> Result<MySqlQueryResult, sqlx::Error> {
        let query_string = "INSERT INTO players (name, rating, deviation, volatility) VALUES (?1, ?2, ?3, ?4)";

        let query = sqlx::query(&query_string)
            .bind(&player.name)
            .bind(player.rating)
            .bind(player.deviation)
            .bind(player.volatility);

        let result = query.execute(&mut **self.inner).await;

        match result {
            Ok(result) => {
                return Ok(result);
            }
            Err(e) => match e {
                _ => {
                    log::error!("Database query failed {} -> {}", query_string, e);
                    panic!("Database query failed");
                }
            },
        }
    }
}
