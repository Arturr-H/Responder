// Go to localhost:8080/index.html or localhost:8080/manual_serve to see the result

/*- Imports -*/
use std::net::TcpStream;
use responder::{*, request::info::Method, response::with_file, response::respond};

/*- Initialize -*/
fn main() {

    /*- Initiaize routes -*/
    let routes = Route::Stack("", &[
        Route::Tail(Method::GET, "manual_serve", Function::S(manual_serve)),
    ]);

    /*- Initiaize server -*/
    start(ServerConfig {
        addr: "127.0.0.1",
        port: 8080u16,
        serve: Some("./examples/static"),
        not_found: None,
        num_threads: 8u16,
        routes,
    }).unwrap();
}

/*- Api endpoints -*/
fn manual_serve(mut stream:&mut TcpStream) -> () {
    /*- Respond with the html file -*/
    respond(
        &mut stream,
        200u16,
        with_file("examples/static/manual.html")
    );
}