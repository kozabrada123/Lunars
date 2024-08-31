use core::panic;

use rocket::form::validate::Len;
use sqlx::mysql::MySqlQueryResult;

use crate::types::entities::{r#match::Match, player::Player};

use super::DbConnection;

impl DbConnection {
    /// Fetches all the matchs.
    // TODO: implement the fancy url parameters we had
	 // (https://github.com/kozabrada123/Lunars/blob/885a8b4ef9ac4f08a45106fe9f5cdc397a81d72c/src/db.rs#L313)
    pub async fn get_matches(&mut self) -> Vec<Match> {
        let query_string = "SELECT * FROM matches";

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

    /// Fetches a match by id
    pub async fn get_match_by_id(&mut self, id: u64) -> Option<Match> {
        let query_string = "SELECT * FROM matches WHERE id = ?1";

        let query = sqlx::query_as(&query_string).bind(id);

        let result: Result<Match, sqlx::Error> = query.fetch_one(&mut **self.inner).await;

        match result {
            Ok(a_match) => {
                return Some(a_match);
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

    /// Updates a match.
    ///
    /// Every field can be changed except id.
    pub async fn modify_match(
        &mut self,
        a_match: &Match,
    ) -> Result<MySqlQueryResult, sqlx::Error> {
        let query_string = "UPDATE matches SET player_a = ?2, player_b = ?3, score_a = ?4, score_b = ?5, ping_a = ?6, ping_b = ?7, rating_a = ?8, rating_b = ?9, deviation_a = ?10, deviation_b = ?11, volatility_a = ?12 volatility_b = ?13, epoch = ?14 WHERE id = ?1";

        let query = sqlx::query(&query_string)
            .bind(a_match.id)
				.bind(a_match.player_a)
				.bind(a_match.player_b)
				.bind(a_match.score_a)
				.bind(a_match.score_b)
				.bind(a_match.ping_a)
				.bind(a_match.ping_b)
				.bind(a_match.rating_a)
				.bind(a_match.rating_b)
				.bind(a_match.deviation_a)
				.bind(a_match.deviation_b)
				.bind(a_match.volatility_a)
				.bind(a_match.volatility_b)
				.bind(a_match.epoch);

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

	 /// Adds a match.
	 ///
	 /// Ignores the id field.
    pub async fn add_match(
        &mut self,
        a_match: &Match,
    ) -> Result<MySqlQueryResult, sqlx::Error> {
        let query_string = "INSERT INTO matches (player_a, player_b, score_a, score_b, ping_a, ping_b, rating_a, rating_b, deviation_a, deviation_b, volatility_a, volatility_b, epoch) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)";

        let query = sqlx::query(&query_string)
				.bind(a_match.player_a)
				.bind(a_match.player_b)
				.bind(a_match.score_a)
				.bind(a_match.score_b)
				.bind(a_match.ping_a)
				.bind(a_match.ping_b)
				.bind(a_match.rating_a)
				.bind(a_match.rating_b)
				.bind(a_match.deviation_a)
				.bind(a_match.deviation_b)
				.bind(a_match.volatility_a)
				.bind(a_match.volatility_b)
				.bind(a_match.epoch);

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
