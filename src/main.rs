extern crate iron;
extern crate router;
extern crate bodyparser;
extern crate hyper_native_tls;
extern crate config;


use iron::prelude::*;
use iron::status;
use router::Router;

use hyper_native_tls::NativeTlsServer;

mod scanner;
mod scan_utils;
mod scan;
mod git;


/// If the server get requests, 
/// call this function and start checking data.
fn serv(req: &mut Request) -> IronResult<Response> {
    // get request's parameter
    let param = req.get::<bodyparser::Json>();

    // parse parameter to json
    let mut messages: Vec<String> = Vec::new();

    let param = match param {
        Ok(Some(_param)) => _param,
        Ok(None) => {
            messages.push("empty".to_string());
            
            let result = scan_utils::ScanResult::init(
                false,
                true,
                messages
            );

            return Ok(Response::with((status::Ok, result.to_string())));
        },
        Err(_err) => {
            messages.push("incorrect json format".to_string());

            let result = scan_utils::ScanResult::init(
                false,
                true,
                messages
            );

            return Ok(Response::with((status::Ok, result.to_string())));
        }
    };

    // make target
    let target = match scan_utils::ScanTarget::new(param) {
        Ok(_target) => _target,
        Err(_) => {
            messages.push("empty".to_string());

            let result = scan_utils::ScanResult::init(
                false,
                true,
                messages
            );

            return Ok(Response::with((status::Ok, result.to_string())));
        }
    };

    // scan
    let result = match scan::scan(target) {
        Ok(_result) => _result,
        Err(_err) => {
            messages.push("unexpected err".to_string());
            
            let result = scan_utils::ScanResult::init(
                false,
                true,
                messages
            );

            result
        }
    };

    // return response
    Ok(Response::with((status::Ok, result.to_string())))
}

/// Set up server and
/// start `git pull` loop
fn main() {
    /* get config */
    // read config file
    let mut settings = config::Config::default();
    match settings.merge(config::File::with_name("conf/Server")) {
        Ok(_) => {},
        Err(_err) => {
            println!("[Err] Setting File Error\n\
                Please check conf/Server.toml exists.\n\n\
                {}", _err);
            return;
        }
    }

    // take out certificate
    let certificate = match settings.get_str("certificate") {
        Ok(_cert) => _cert,
        Err(_err) => {
            println!("[Err] Certificate Option Error\n\
                Please check conf/Server.toml has the PATH of certificate file.");
            return;
        }
    };

    // take out password
    let password = match settings.get_str("password") {
        Ok(_pass) => _pass,
        Err(_err) => {
            println!("[Err] Password Option Error\n\
                Please check conf/Server.toml has the password.");
            return;
        }
    };

    // take out password
    let address = match settings.get_str("address") {
        Ok(_address) => _address,
        Err(_err) => {
            println!("[Err] Address Option Error\n\
                Please check conf/Server.toml has the address of server.");
            return;
        }
    };

    /* get ssl certification */
    let ssl = match NativeTlsServer::new(certificate, &password) {
        Ok(_ssl) => _ssl,
        Err(_err) => {
            println!("[Unexpected Err] Unexpected SSL Error\n\
                Language Error:{}\n\
                (Maybe... : Is the identity file exists ?)", _err);
            return;
        }
    };
    
    git::auto_pull();

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
