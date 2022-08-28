// Go to localhost:8080 to see the result

/*- Imports -*/
use std::net::TcpStream;
use responder::{*, request::info::Method, response::Respond, response::respond};
use rand::Rng;

/*- Initialize -*/
fn main() {

    /*- Initiaize routes -*/
    let routes = Route::Stack("", &[
        Route::Tail(Method::GET, "", Function::S(respond_with_random_number)),
    ]);

    /*- Initiaize server -*/
    start(ServerConfig {
        addr: "127.0.0.1",
        port: 8080u16,
        serve: None,
        not_found: None,
        routes,
    }).unwrap();
}

/*- Api endpoints -*/
fn respond_with_random_number(mut stream:&mut TcpStream) -> () {
    /*- Get random number -*/
    let random_number = rand::thread_rng().gen_range(0..100);

    /*- Respond with the random number -*/
    respond(
        &mut stream,
        200u16,
        Respond::text(
            &format!(
                "Random number between 0 and 100: {}",
                &random_number
            )
        )
    );
}