// Go to localhost:8080/post

/*- Imports -*/
use std::{net::TcpStream, io::Write};
use responder::{*, request::info::Method, response::respond};
// use std::fs;

/*- Initialize -*/
fn main() {

    /*- Initiaize routes -*/
    let routes = Route::Stack("", &[
        Route::Tail(Method::POST, "post", Function::SB(upload_image))
    ]);

    /*- Initiaize server -*/
    Server::new()
        .routes(routes)
        .address("127.0.0.1")
        .port(8080)
        .start()
        .unwrap();
}

fn upload_image(stream:&mut TcpStream, body:&String) -> () {
    println!("online");
    let mut file = std::fs::File::create("./examples/static/uploaded.png").unwrap();
    file.write_all(body.as_bytes()).unwrap();
    respond(stream, 200u16, None);
}