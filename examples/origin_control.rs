// Go to localhost:8080

/*- Imports -*/
use std::net::TcpStream;
use responder::{ *, response::respond, request::info::Method };

/*- Initialize -*/
fn main() {

    /*- Initiaize routes -*/
    let route = Route::Stack("", &[
        Route::Tail(Method::GET, "", Function::S(test))
    ]);

    /*- Initiaize server -*/
    Server::new()
        .routes(route)
        .address("127.0.0.1")
        .origin_control(|stream, headers| {
            match headers.get("Host") {
                Some(host) => {
                    if host == &"" {
                        respond(stream, 401u16, None);
                        true
                    }else {
                        false
                    }
                },
                None => {
                    respond(stream, 401u16, None);
                    true
                }
            }
        })
        .port(8080)
        .start()
        .unwrap();
}

fn test(stream:&mut TcpStream) -> () {
    respond(stream, 200u16, None);
}