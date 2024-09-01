use core::panic;

use sqlx::mysql::MySqlQueryResult;

use crate::types::entities::r#match::Match;

use super::{query::QueryParameters, DbConnection};

impl DbConnection {
    /// Fetches all the matches.
    ///
    /// Use query_parameters to set order_by, max, min, ...
    pub async fn get_matches(&mut self, query_parameters: QueryParameters) -> Vec<Match> {
        let query_string = "SELECT * FROM matches";

        let (query_string, parameters) = self.add_to_query(query_string, query_parameters).await;

        let mut query = sqlx::query_as(&query_string);

        for parameter in parameters {
            query = query.bind(parameter);
        }

        let result = query.fetch_all(&mut **self.inner).await;

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
        let query_string = "SELECT * FROM matches WHERE id = ?";

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

    /// Fetches a player's matches, by their id
    pub async fn get_player_matches(&mut self, id: u64) -> Vec<Match> {
        let query_string = "SELECT * FROM matches WHERE (player_a = ? OR player_b = ?)";

        let query = sqlx::query_as(&query_string).bind(id).bind(id);

        let result: Result<Vec<Match>, sqlx::Error> = query.fetch_all(&mut **self.inner).await;

        match result {
            Ok(matches) => {
                return matches;
            }
            Err(e) => match e {
                sqlx::Error::RowNotFound => return Vec::new(),
                _ => {
                    log::error!("Database query failed {} -> {}", query_string, e);
                    panic!("Database query failed");
                }
            },
        }
    }

    /// Fetches a player's matches, by their id, for a specific season
    pub async fn get_player_matches_for_season(&mut self, id: u64, season: u64) -> Vec<Match> {
        let query_string =
            "SELECT * FROM matches WHERE (player_a = ? OR player_b = ?) AND season = ?";

        let query = sqlx::query_as(&query_string).bind(id).bind(id).bind(season);

        let result: Result<Vec<Match>, sqlx::Error> = query.fetch_all(&mut **self.inner).await;

        match result {
            Ok(matches) => {
                return matches;
            }
            Err(e) => match e {
                sqlx::Error::RowNotFound => return Vec::new(),
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
    pub async fn modify_match(&mut self, a_match: &Match) -> Result<MySqlQueryResult, sqlx::Error> {
        let query_string = "UPDATE matches SET rating_period = ?, player_a = ?, player_b = ?, score_a = ?, score_b = ?, ping_a = ?, ping_b = ?, rating_a = ?, rating_b = ?, deviation_a = ?, deviation_b = ?, volatility_a = ? volatility_b = ?, epoch = ? WHERE id = ?";

        let query = sqlx::query(&query_string)
            .bind(a_match.rating_period)
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
            .bind(a_match.epoch)
            .bind(a_match.id);

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
    pub async fn add_match(&mut self, a_match: &Match) -> Result<MySqlQueryResult, sqlx::Error> {
        let query_string = "INSERT INTO matches (rating_period, player_a, player_b, score_a, score_b, ping_a, ping_b, rating_a, rating_b, deviation_a, deviation_b, volatility_a, volatility_b, epoch) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)";

        let query = sqlx::query(&query_string)
            .bind(a_match.rating_period)
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
