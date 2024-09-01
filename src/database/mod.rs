use rocket::{
    fairing::{self, AdHoc},
    Build, Rocket,
};
use rocket_db_pools::{Connection, Database};

use log::error;

use crate::MysqlDb;
pub mod r#match;
pub mod player;
pub mod query;
pub mod season;

pub struct DbConnection {
    pub inner: Connection<MysqlDb>,
}

impl DbConnection {
    pub fn from_inner(inner: Connection<MysqlDb>) -> Self {
        Self { inner }
    }
}

async fn run_migrations(rocket: Rocket<Build>) -> fairing::Result {
    match MysqlDb::fetch(&rocket) {
        Some(db) => match sqlx::migrate!().run(&**db).await {
            Ok(_) => Ok(rocket),
            Err(e) => {
                error!("Failed to initialize SQLx database: {}", e);
                Err(rocket)
            }
        },
        None => Err(rocket),
    }
}

/// Database creation "fairing"
pub fn stage() -> AdHoc {
    AdHoc::on_ignite("SQLx Stage", |rocket| async {
        rocket
            .attach(MysqlDb::init())
            .attach(AdHoc::try_on_ignite("SQLx Migrations", run_migrations))
    })
}
