// Main file: Hosts server that connects to database.
// Make sure to first configure .env and your json file with key hashes.

// -----------------------

// Imports
// -----------------------
#[macro_use]
extern crate nickel;
extern crate dotenv;
extern crate serde;

use dotenv::dotenv;
use flexi_logger::{colored_detailed_format, Duplicate, FileSpec, Logger, WriteMode};
use log::{info};
use nickel::{HttpRouter, Nickel};
use regex::Regex;
use std::{env, thread, time};

mod calculations;
mod db;
mod middlewares;
// -----------------------

fn main() {
    // Make db if not exists
    db::DbConnection::new().setup();

    // Load .env file
    dotenv().ok();

    // Init logger
    let _logger = Logger::try_with_str("info, lunars=trace")
        .unwrap()
        .log_to_file(FileSpec::default().directory(env::var("LOGDIR").unwrap()))
        .duplicate_to_stdout(Duplicate::All)
        .format_for_stdout(colored_detailed_format)
        .write_mode(WriteMode::BufferAndFlush)
        .start()
        .unwrap();

    // Init beautiful art into the log
    log_logo();

    // Make a nickel server
    let mut server = Nickel::new();

    // Server paths
    // Regex path for players so dots work
    let players_api_regex = Regex::new("/api/players/(?P<query>[A-Za-z0-9_.-]{4,24})").unwrap();

    // Gets players
    server.get(
        "/api/players",
        middleware! { |request, mut response|

        // Only calls getPlayers, look there
        let responsedata = middlewares::get_players(request, &mut response);

        responsedata
        },
    );

    // Gets a player
    server.get(
        players_api_regex,
        middleware! { |request, mut response|

            // Only calls getPlayer, look there
            let responsedata = middlewares::get_player(request, &mut response);

            responsedata

        },
    );

    // Gets matches
    server.get(
        "/api/matches",
        middleware! { |request, mut response|

        // Only calls getMatches, look there
        let responsedata = middlewares::get_matches(request, &mut response);

        responsedata

        },
    );

    // Gets a match
    server.get(
        "/api/matches/:query",
        middleware! { |request, mut response|

            // Only calls getMatch, look there
            let responsedata = middlewares::get_match(request, &mut response);

            responsedata

        },
    );

    // Adds a player
    server.post(
        "/api/players/add",
        middleware! { |request, mut response|

            // Only calls addPlayer, look there
            let responsedata = middlewares::add_player(request, &mut response);

            responsedata

        },
    );

    // Submits a match
    server.post(
        "/api/matches/add",
        middleware! { |request, mut response|

        // Only calls addMatch, look there
        let responsedata = middlewares::add_match(request, &mut response);

        responsedata

        },
    );

    // Calculates a dummy match, DOES NOT UPDATE RECORDS!
    server.post(
        "/api/matches/dummy",
        middleware! { |request, mut response|

            // Only calls testMaatch, look there
            let responsedata = middlewares::test_match(request, &mut response);

            responsedata

        },
    );

    // Create a backup thread so that I dont fuck up production
    thread::spawn(move || {

        loop {
            // Wait a day and then backup
            thread::sleep(
                time::Duration::from_secs(10) // 1 day
            );
            
            db::backup();

        }
    });

    server.listen("0.0.0.0:6767").unwrap();
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
