use core::panic;

use log::info;
use rocket::form::validate::Len;
use sqlx::mysql::MySqlQueryResult;

use crate::types::entities::player::Player;

use super::{query::QueryParameters, DbConnection};

impl DbConnection {
    /// Fetches all the players.
    ///
    /// Use query_parameters to set order_by, max, min, ...
    pub async fn get_players(&mut self, query_parameters: QueryParameters) -> Vec<Player> {
        let query_string = "SELECT * FROM players";

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

    /// Fetches all players with a similar username to the input string.
    ///
    /// Also uses query_parameters to set order_by, max, min, ...
    pub async fn search_players(
        &mut self,
        search_string: &str,
        query_parameters: QueryParameters,
    ) -> Vec<Player> {
        let query_string = "SELECT * FROM players WHERE name LIKE ?";

        let (query_string, parameters) = self.add_to_query(query_string, query_parameters).await;

        let mut query = sqlx::query_as(&query_string).bind(format!("%{search_string}%"));

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

    /// Fetches a player by name
    pub async fn get_player_by_name(&mut self, name: &str) -> Option<Player> {
        let query_string = "SELECT * FROM players WHERE name = ?";

        let query = sqlx::query_as(&query_string).bind(name);

        let result: Result<Vec<Player>, sqlx::Error> = query.fetch_all(&mut **self.inner).await;

        match result {
            Ok(vec) => {
                if vec.len() < 1 {
                    return None;
                }

                if vec.len() > 1 {
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
        let query_string = "SELECT * FROM players WHERE id = ?";

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

    /// Fetches a player by id or name
    ///
    /// Id takes priority over name
    // TODO: Impose a restriction that player's name could not ever be ids
    pub async fn get_player_by_id_or_name(&mut self, query: &str) -> Option<Player> {
        let as_u64_res = query.parse::<u64>();

        if let Ok(as_64) = as_u64_res {
            let by_id = self.get_player_by_id(as_64).await;

            if let Some(player) = by_id {
                return Some(player);
            }
        }

        let as_name = self.get_player_by_name(query).await;

        if let Some(player) = as_name {
            return Some(player);
        }

        None
    }

    /// Updates a player.
    ///
    /// Every field can be changed except id.
    pub async fn modify_player(
        &mut self,
        player: &Player,
    ) -> Result<MySqlQueryResult, sqlx::Error> {
        let query_string =
            "UPDATE players SET name = ?, rating = ?, deviation = ?, volatility = ? WHERE id = ?";

        let query = sqlx::query(&query_string)
            .bind(&player.name)
            .bind(player.rating)
            .bind(player.deviation)
            .bind(player.volatility)
            .bind(player.id);

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
    pub async fn add_player(&mut self, player: &Player) -> Result<MySqlQueryResult, sqlx::Error> {
        let query_string =
            "INSERT INTO players (name, rating, deviation, volatility) VALUES (?, ?, ?, ?)";

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
