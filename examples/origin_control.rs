// Go to localhost:8080

/*- Imports -*/
use responder::prelude::*;

/*- Initialize -*/
fn main() {

    /*- Initiaize routes -*/
    let routes = &[
        /*- All Route enum variants inside of this `ControlledStack` will
            only be reachable if the origin control function returns true -*/
        Route::ControlledStack(origin_control_function, "", &[
            Route::Get("", test)
        ])
    ];

    /*- Initiaize server -*/
    Server::new()
        .routes(routes)
        .address("127.0.0.1")
        .port(8080)
        .start()
        .unwrap();
}

/*- GET endpoint inside of the controlled stack -*/
fn test(stream:&mut Stream) -> () {
    stream.respond(200u16, Respond::new().text(r#"You have the "Host" header!"#));
}

/*- If the request has the header "Host", we accept
    the request, otherwise we'll cancel it -*/
fn origin_control_function(stream:&mut Stream) -> bool {
    match stream.headers.get("Host") {
        Some(host) => {
            /*- One important think to keep in mind is that this origin control is
                very weak and easy to bypass. You will need to implement more than
                just an empty-string checker for better backend-security -*/
            if host == &"" {
                /*- Respond with unauthorized status code, and
                    return false indicating that the request failed -*/
                stream.respond_status(401);
                false
            }else {
                /*- Return true indicating that the request should continue.
                    We don't use any stream methods here because they
                    will be used later in the endpoint functions -*/
                true
            }
        },
        None => {
            /*- Respond with unauthorized status code, and
                return false indicating that the request failed -*/
            stream.respond_status(401);
            false
        }
    }
}
