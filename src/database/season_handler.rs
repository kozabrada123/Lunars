use std::collections::HashSet;

use chrono::Utc;
use log::info;
use rocket::form::validate::Len;

use crate::{
    glicko,
    types::entities::{player::Player, r#match::Match, season::Season},
    MysqlDb,
};

/// Initializes the season handler, creates an active season
/// if there isn't one, starts the season update task
pub async fn initialize_season_handler(db: &MysqlDb) {
    let now = Utc::now();

    let query = sqlx::query_as(
        "SELECT * FROM rating_periods WHERE (start < ? AND end > ?) ORDER BY id DESC LIMIT 1",
    )
    .bind(now)
    .bind(now);

    let active_season_result: Result<Season, sqlx::Error> = query.fetch_one(&**db).await;

    let active_season = match active_season_result {
        Ok(season) => season,
        Err(e) => match e {
            sqlx::Error::RowNotFound => create_new_season(db).await,
            _ => {
                log::error!("Seasons handler: Encountered database error: {}", e);
                panic!("Seasons handler encountered database error");
            }
        },
    };

    let db_clone = db.clone();

    tokio::spawn(async move {
        season_handler_main_task(db_clone, active_season).await;
    });
}

/// Main loop of the season handler;
///
/// Wait until the end of this season, update all player
/// ranks, create new season
pub async fn season_handler_main_task(db: MysqlDb, season: Season) {
    let mut active_season = season;

    loop {
        let now = Utc::now();

        if now < active_season.end {
            let to_wait = active_season.end - now;

            info!(
                "Seasons handler: Waiting for end of season at {} -- {}",
                active_season.end, to_wait
            );

            tokio::time::sleep(to_wait.to_std().unwrap()).await;
        }

        info!(
            "Seasons handler: Season {} has come to a close!",
            active_season.id
        );

        rate_all_players_for_season(&db, &active_season).await;

        active_season = create_new_season(&db).await;
    }
}

/// Creates an returns a new season, starting now
pub async fn create_new_season(db: &MysqlDb) -> Season {
    let now = Utc::now();
    let end = now + glicko::RATING_PERIOD_DURATION;
    let mut new_season = Season {
        start: now,
        end,
        id: 0,
    };

    let query = sqlx::query("INSERT INTO rating_periods (start, end) VALUES (?, ?)")
        .bind(new_season.start)
        .bind(new_season.end);

    let result = query.execute(&**db).await;

    let season_id = result.unwrap().last_insert_id();

    info!("Season handler: Created new season (id {})!", season_id);

    new_season.id = season_id;

    new_season
}

/// Concludes a season and writes updated player rankings
pub async fn rate_all_players_for_season(db: &MysqlDb, season: &Season) {
    let start = std::time::Instant::now();

    let query = sqlx::query_as("SELECT * FROM matches WHERE rating_period = ?").bind(season.id);

    let result: Result<Vec<Match>, sqlx::Error> = query.fetch_all(&**db).await;

    if let Err(e) = result.as_ref() {
        log::error!("Seasons handler: Failed to get season matches! {}", e);
    }

    let season_matches = result.unwrap();

    let query = sqlx::query_as("SELECT * FROM players");

    let result: Result<Vec<Player>, sqlx::Error> = query.fetch_all(&**db).await;

    if let Err(e) = result.as_ref() {
        log::error!("Seasons handler: Failed to get players! {}", e);
    }

    let mut players = result.unwrap();

    // Go through each player, find their matches, compute their rating, update it
    for player in &mut players {
        let player_matches = season_matches
            .iter()
            .filter(|a_match| a_match.player_a == player.id || a_match.player_b == player.id)
            .cloned()
            .collect::<Vec<Match>>();

        // Note: should we use a computed completion here or just 1.0?
        player.rate_player_for_elapsed_periods(player_matches, 1.0);

        let query = sqlx::query(
            "UPDATE players SET rating = ?, deviation = ?, volatility = ? WHERE id = ?",
        )
        .bind(player.rating)
        .bind(player.deviation)
        .bind(player.volatility)
        .bind(player.id);

        let result = query.execute(&**db).await;

        match result {
            Ok(_) => {}
            Err(e) => {
                log::error!(
                    "Seasons handler: Failed to update player {} for end of season! {}",
                    player.id,
                    e
                );
                continue;
            }
        }
    }

    let elapsed = start.elapsed();

    log::info!("Seasons handler: computed and saved ratings for season {} - {} players and {} matches - took {:?}", season.id, players.len(), season_matches.len(), elapsed);
}
