extern crate iron;
extern crate router;
extern crate bodyparser;
extern crate hyper_native_tls;
extern crate config;


use std::collections::HashMap;
use std::string::String;

use iron::prelude::*;
use iron::status;
use router::Router;

use hyper_native_tls::NativeTlsServer;


/// If the server get requests, 
/// call this function and start checking data.
fn serv(req: &mut Request) -> IronResult<Response> {
    // get request's parameter
    let param = req.get::<bodyparser::Json>();
    println!("Parsed body:\n{:?}", param);

    // parse parameter to json
    let result = match param {
        Ok(Some(_param)) => "ok",
        Ok(None) => "empty",
        Err(_err) => "error"
    };

    // return response
    Ok(Response::with((status::Ok, result)))
}

/// Set up server and
/// start `git pull` loop
fn main() {
    // get config
    let mut settings = config::Config::default();
    settings
        .merge(config::File::with_name("Settings")).unwrap();

    let setting_data = settings.deserialize::<HashMap<String, String>>().unwrap();
    let certificate = setting_data.get("certificate").unwrap();
    let password = setting_data.get("password").unwrap();
    let address = setting_data.get("address").unwrap();

    // get ssl certification
    let ssl = NativeTlsServer::new(certificate, password).unwrap();
    println!("On {}", address);

    // set up server
    let mut router = Router::new();
    router.post("/", serv, "serv");
    Iron::new(router).https(address, ssl).unwrap();
}
