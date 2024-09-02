// Main file: Hosts server that connects to database.
// Make sure to first configure .env and your json file with key hashes.

use std::path::PathBuf;

use dotenv::dotenv;
use log::info;
use rocket::{catchers, fairing::AdHoc};
use rocket_cors::CorsOptions;
use rocket_db_pools::Database;

use rocket_okapi::{
    openapi_get_routes,
    swagger_ui::{make_swagger_ui, SwaggerUIConfig},
};
use simplelog::{TermLogger, WriteLogger};

mod calculations;
mod database;
mod glicko;
mod request_guards;
mod response;
mod routes;
mod types;

use routes::{
    catchers::default_catcher,
    matches::{add::*, get::*},
    players::{add::*, get::*},
    system::get_constants::*,
    system::seasons::get::*,
};

#[derive(Database, Debug, Clone)]
#[database("mysql")]
struct MysqlDb(sqlx::MySqlPool);

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {
    dotenv().expect("No .env file found!");

    let logdir = std::env::var("LOGDIR").unwrap_or("/usr/src/Lunars/logs".to_string());

    let mut path = PathBuf::from(logdir);
    path.push("latest.log");

    let log_file = std::fs::File::create(path).unwrap();

    simplelog::CombinedLogger::init(vec![
        TermLogger::new(
            log::LevelFilter::Info,
            simplelog::Config::default(),
            simplelog::TerminalMode::Mixed,
            simplelog::ColorChoice::Auto,
        ),
        WriteLogger::new(
            log::LevelFilter::Info,
            simplelog::Config::default(),
            log_file,
        ),
    ])
    .unwrap();

    let _rocket = rocket::build()
        .attach(AdHoc::on_liftoff("Necessary log", |_rocket| {
            Box::pin(async { log_logo() })
        }))
        .attach(database::stage())
        .attach(CorsOptions::default().to_cors().unwrap())
        .mount(
            "/swagger-ui/",
            make_swagger_ui(&SwaggerUIConfig {
                url: "../openapi.json".to_owned(),
                ..Default::default()
            }),
        )
        .register("/", catchers![default_catcher])
        .mount(
            "/",
            openapi_get_routes![
                get_players,
                get_players_live,
                get_player,
                get_player_live,
                add_player,
                search_players,
                get_matches,
                get_match,
                add_match,
                get_seasons,
                get_season,
                get_latest_season,
                get_system_constants
            ],
        )
        .launch()
        .await?;

    Ok(())
}

// 100% required feature ( ͡° ͜ʖ ͡°)
// There is a better way to do this, like using a text file or converting but for some reason I dont want to do that
fn log_logo() {
    info!("");
    info!("                   &##&                        5?& JJ                       &#B#&                   ");
    info!("                &#BB#                        &J~~7?~~?&                       &BBB#&                ");
    info!("           && &BBBB&                   #GPYJ?!~~~~~~~~!?JYPG#&                  #BBBB&&#&           ");
    info!("         #BB&#BBB#                &B5?!~~~~^^~~~~~~~~~~^^~~~~!?5B&               &#BBB&&BB&         ");
    info!("        #BB&#BB#&#&             GJ!~~~~~~~!7?J?~~~~~~7J?7!~~~~~~~!JG&            #&&#BB #BB#        ");
    info!("       #BB# #&&#B#           &57~~~~~~7YP#&    ?~~~~~#   &#GY?!~~~~~!Y#          &BB#&#& BBB&#&     ");
    info!("    ## BBB &#BBB#          &Y!~~~~~?P#         ?~~~~~#        &GJ!~~~~~Y#          #BBB#&#BB#&B&    ");
    info!("   #B&&BB##BBB#           P!~~~~~J#            ?~~~~~#           &5!~~~~~5           #BBBBBB# BB&   ");
    info!("  &BB#&BBBB#&#          &?~~~~~J#              ?~~~~~#             &5!~~~~?#         ##&#BBB# BBB   ");
    info!("  #BB#&BB&&#B#         #7~~~~!B                J~~~~~&               #?~~~~!#        &BB&&&B# BBB&  ");
    info!("  #BBB&&&#BBB         &7~~~~7#              &GP!~~~~~YPB               J~~~~!#        &BBB#& &BBB#  ");
    info!("  #BBB&&BBB#          J~~~~!&               5^~~~~~~~~~~#               J~~~~?          #BBB##BBB&& ");
    info!("#& BBBBBBB&          G~~~~~B            &##G!~~~~~~~~~~~?B#&            &!~~~~P         &&BBBBBB# B#");
    info!("B# #BBBB# ##       #G7~~~~!&           G!~~~~~~~~~~~~~~~~~~~?            J~~~~7G#       B& #BBBB &BB");
    info!("BB& BBB& #B#     G?~~~~~~~~7???????????!~~~~~~~~~~~~~~~~~~~~~7??????????7~~~~~~~~?P     #B# &BB& BBB");
    info!("BBB& B# #BB&     &G?~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~?G&     &BBB &# #BB#");
    info!("#BBB&  BBB#      G?~~~~~~~~!77777777777~~~~~~~~~~~~~~~~~~~~~~!7777777777!~~~~~~~~7P      #BBB  #BBB&");
    info!(" BBBB #BBB         #P7~~~~!#           5~~~~~~~~~~~~~~~~~~~~7&           ?~~~~!5B&        BBB##BBB# ");
    info!("  BBB#BBB&&#         G~~~~~B            BGPY~~~~~~~~~~~~!5PG#           &7~~~~P        &# &BBBBBB# &");
    info!("B& #BBBBB #B&         ?~~~~7&               Y^~~~~~~~~~~B               J~~~~7         #B& #BBBB& #B");
    info!("BB& &BBB& BB#         #!~~~~?&              &5Y!~~~~~JYP               Y~~~~!#         BB# #BB# &BB#");
    info!("&BBB&&#B& BBB          #!~~~~7B                ?~~~~~&               &J~~~~!B         &BBB &#&&#BB# ");
    info!(" &BBBB&& &BBB&          #?~~~~~Y&              ?~~~~~#              P!~~~~7#        & &BBB  &#BBB#  ");
    info!("  &#BBBB&&BBB&&B&         5~~~~~!Y#            ?~~~~~#           &P7~~~~~Y        &BB #BBB&#BBBB&   ");
    info!("    &#BBBBBBB# #B#         #J~~~~~~JG&         ?~~~~~#        &BY!~~~~~J#        &BB# BBBBBBB#&     ");
    info!("    #&&#BBBBBB &BB#          #Y!~~~~~!?5G#&    ?~~~~~&    &B5?!~~~~~!Y#         &BBB  BBBBB#& ##    ");
    info!("    &B#&&&&#BB# BBB#           &GJ!~~~~~~~!7JYJ!~~~~~?JJ?7~~~~~~~!?P&           BBB# #B#&&&&#B#     ");
    info!("     &BBB##&&&# &BBB# &           &GY?!~~~~^^~~~~~~~~~~^^~~~~!?YG&          && #BBB  &&&#BBBB&      ");
    info!("       &#BBBB##& &BBB&&B#&            &#G5J?7!~~~~~~~~!7?J5GB&            &BB&#BBB&&##BBBBB&        ");
    info!("         &##BBBBB#BBBB&&BBB&                 &?~~77~~?#                 #BBB&&BBBBBBBBB##&          ");
    info!("            &&&##BBBBBB&&BBBB&                 Y?&&?Y                &#BBB#&#BBBBB##&&&&&           ");
    info!("            &##&&&&&&###& &#BBB#&                                  &#BBB#& &#&&&&&&##BB&            ");
    info!("              #BBBBBBBBBBB##BBBBBBB##&  &&&&#&&&    &&###&&&   ##BBBBBBBBBBBBBBBBBBB#&              ");
    info!("                 &&###BBBBBB##########BBBBB###BBB##BBB####BBBB####&&############&&                  ");
    info!("                    &&&&&   &&&##BBBBBB#&&&##B#&&   &#B##& &#BBBBBB###&&&&&&&&#&                    ");
    info!("                     &##BBBBBBBBBBBB#& &#BBB&          &BBB#&&&#BBBBBBBBBBBB##&                     ");
    info!("                         &&#####&&&  &#BB#&              &#BBB&   &&&&&&&&                          ");
    info!("                                     &B#&                  &#B#                                     ");
    info!("");
    info!(
        "{}",
        format!("{}: v{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    );
    info!("");
}
