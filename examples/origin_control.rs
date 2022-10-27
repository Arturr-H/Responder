// Go to localhost:8080

/*- Imports -*/
use responder::{ *, stream::Stream, response::Respond };

/*- Initialize -*/
fn main() {

    /*- Initiaize routes -*/
    let route = Route::ControlledStack(origin_control_function, "", &[
        Route::Get("", test)
    ]);

    /*- Initiaize server -*/
    Server::new()
        .routes(route)
        .address("127.0.0.1")
        .origin_control(origin_control_function)
        .port(8080)
        .start()
        .unwrap();
}

fn test(stream:&mut Stream) -> () {
    stream.respond(200u16, Respond::new().text("Hello, world!"));
}

fn origin_control_function(stream:&Stream) -> Result<(), u16> {
    match stream.headers.get("Host") {
        Some(host) => {
            if host == &"" {
                Err(401)
            }else {
                Ok(())
            }
        },
        None => {
            Err(401)
        }
    }
}