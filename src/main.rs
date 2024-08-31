// Main file: Hosts server that connects to database.
// Make sure to first configure .env and your json file with key hashes.

use dotenv::dotenv;
use log::info;
use rocket::routes;
use rocket_cors::CorsOptions;
use rocket_db_pools::Database;
use routes::players::get::*;

mod calculations;
mod glicko;
mod database;
mod routes;
mod types;
mod response;
mod request_guards;

#[derive(Database, Debug, Clone)]
#[database("mysql")]
struct MysqlDb(sqlx::MySqlPool);

#[rocket::main]
async fn main() -> Result<(), rocket::Error> {

	dotenv().expect("No .env file found!");

	let _rocket = rocket::build()
		.attach(database::stage())
		.attach(CorsOptions::default().to_cors().unwrap())
		.mount("/players", routes![get_players])
		.launch().await?;

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
