extern crate iron;
extern crate router;
extern crate bodyparser;

use iron::prelude::*;
use iron::status;
use router::Router;


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
    println!("On 3000");

    // set up server
    let mut router = Router::new();
    router.post("/", serv, "serv");
    Iron::new(router).http("localhost:3000").unwrap();
}
