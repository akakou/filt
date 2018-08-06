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

mod scan;


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
    // scan
    scan::scan();
    
    /* get config */
    // read config file
    let mut settings = config::Config::default();
    match settings.merge(config::File::with_name("conf/Server")) {
        Ok(_) => {},
        Err(_err) => {
            println!("[Err] Setting File Error\n\
                Please check Settings.toml exists.\n\n\
                {}", _err);
            return;
        }
    }

    // convert to hashmap
    let setting_data = match settings.try_into::<HashMap<String, String>>() {
        Ok(_setting) => _setting,
        Err(_err) => {
            println!("[Err] Setting Data Error\n\
                Please check Setting.toml correct.\n\n\
                {}", _err);
            return;
        }
    };

    // take out certificate
    let certificate = match setting_data.get("certificate") {
        Some(_cert) => _cert,
        None => {
            println!("[Err] Certificate Option Error\n\
                Please check Setting.toml has the PATH of certificate file.");
            return;
        }
    };

    // take out password
    let password = match setting_data.get("password") {
        Some(_pass) => _pass,
        None => {
            println!("[Err] Password Option Error\n\
                Please check Setting.toml has the password.");
            return;
        }
    };

    // take out password
    let address = match setting_data.get("address") {
        Some(_address) => _address,
        None => {
            println!("[Err] Address Option Error\n\
                Please check Setting.toml has the address of server.");
            return;
        }
    };

    /* get ssl certification */
    let ssl = match NativeTlsServer::new(certificate, password) {
        Ok(_ssl) => _ssl,
        Err(_err) => {
            println!("[Unexpected Err] Unexpected SSL Error\n\
                Language Error:{}\n\
                (Maybe... : Is the identity file exists ?)", _err);
            return;
        }
    };
    
    /* run server */
    println!("On {}", address);
    
    let mut router = Router::new();
    router.post("/", serv, "serv");
    match Iron::new(router).https(address, ssl) {
        Ok(_) => {},
        Err(_err) => {
            println!("[Unexpected Err] Unexpected Server Error\n\
                {}", _err)
        }
    };
}
