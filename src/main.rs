#[macro_use] 
extern crate nickel;
extern crate serde;
extern crate dotenv;

use std::{fs, any::type_name};
use dotenv::dotenv;
use nickel::{Nickel, HttpRouter, JsonBody, Response};
use nickel::status::StatusCode::{Forbidden, ImATeapot};
use serde::{Serialize, Deserialize};
use sha256::digest;
use serde_json;

// Struct of the valid authentication keys
// TODO: add perms
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Clone)]
struct AuthKey {
    hash : String,
}

impl AuthKey {
    fn new(hash: &str) -> AuthKey {
        AuthKey { hash: hash.to_string() }
    }
}

// Struct of what we get in /add
#[derive(Serialize, Deserialize, Debug)]
struct AddStruct {
    token : String,
    user_a : String,
    ping_a : u16,
    score_a : u16,
    user_b : String,
    ping_b : u16,
    score_b : u16,
}

fn main() {
    // Load .env file
    dotenv().ok();

    let mut server = Nickel::new();

    // Server paths
    server.get("/get/", middleware! {
        // We don't need a key to get elo and matches
        ("/get/")
    });

    server.get("/add/", middleware! { |request, mut response| response.set(ImATeapot); "Invalid method. Please use POST."});

    server.post("/add/", middleware! { |request, mut response|
        // What we'll send in response
        let mut responsedata = "";

        // Authenticate with json string key
        let parameters = request.json_as::<AddStruct>().unwrap();
        // pass to authenticator
        let authenticated = authenticator(parameters.token);

        if !authenticated {
            // Not authenticated, get forbidden
            response.set(Forbidden);
            responsedata = "Invalid Token"
        }

        else {
            //TODO: Do database stuff here
            responsedata = "Congrats! We haven't written the database code yet! But you do have a valid key"
        }

        format!("{}", responsedata)

    });

    server.listen("127.0.0.1:6767").unwrap();
}

fn authenticator(key: String) -> bool {
    // Authenticator with sha256 keys
    // TODO: Make this better somehow so we aren't saving stuff in json
    // Maybe add it into the database? Not sure how much I like that however..

    // See which file has our keys
    dotenv().ok(); // Load env
    let authfile = std::env::var("KEYFILE").unwrap();

    // Read our raw key data
    let raw_keys = fs::read_to_string(authfile)
        .expect("Couldn't read keyfile");

    // Parse it in json
    let keys: Vec<AuthKey> = serde_json::from_str(&raw_keys).unwrap();

    // Process sha256 key hash
    let key_hash = digest(key);

    // Return whether or not the know the hash
    // We use a for loop here because if we had AuthKey(key_hash) and permissions we would get false
    // Remember, we are checking if we know the key, not if we know the exact permissions thing
    // For now...
    // Eventually we'll want to check if we have the right permissions
    for AuthKey in keys {
        if AuthKey.hash == key_hash {return true}
    }

    false
}