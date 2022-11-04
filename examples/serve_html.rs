// Go to localhost:8080/index.html or localhost:8080/manual_serve to see the result

/*- Imports -*/
use responder::prelude::*;

/*- Initialize -*/
fn main() {

    /*- Initiaize routes -*/
    let routes = &[
        Route::Get("manual_serve", manual_serve),
    ];

    /*- Initiaize server -*/
    Server::new()
        .routes(routes)
        .address("127.0.0.1")
        .serve("./examples/static")
        .port(8080)
        .start()
        .unwrap();
}

/*- Api endpoints -*/
fn manual_serve(stream:&mut Stream) -> () {
    /*- Respond with the html file -*/
    stream.respond(
        200u16,
        with_file("examples/static/manual.html")
    );
}