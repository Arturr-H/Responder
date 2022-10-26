// Go to localhost:8080

/*- Imports -*/
use responder::{ *, request::info::Method, stream::Stream, response::Respond };

/*- Initialize -*/
fn main() {

    /*- Initiaize routes -*/
    let route = Route::Stack("", &[
        Route::Tail(Method::GET, "", test)
    ]);

    /*- Initiaize server -*/
    Server::new()
        .routes(route)
        .address("127.0.0.1")
        .origin_control(|stream, headers| {
            match headers.get("Host") {
                Some(host) => {
                    if host == &"" {
                        stream.respond_status(401);
                        true
                    }else {
                        false
                    }
                },
                None => {
                    stream.respond_status(401);
                    true
                }
            }
        })
        .port(8080)
        .start()
        .unwrap();
}

fn test(stream:&mut Stream) -> () {
    stream.respond(200u16, Respond::new().text("Hello, world!"));
}