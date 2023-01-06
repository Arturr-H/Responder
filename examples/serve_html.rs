// Go to localhost:8080/index.html, localhost:8080/manual_serve_file or localhost:8080/manual_serve to see the result

/*- Imports -*/
use responder::prelude::*;

/*- Initialize -*/
fn main() {
    /*- Initiaize routes -*/
    let routes = &[
        Route::File(
            "manual_serve_file",
            "./examples/static/manual_serve_file.html",
        ),
        Route::Get("manual_serve", manual_serve),
    ];

    /*- Initiaize server -*/
    Server::new()
        .routes(routes)
        .address("127.0.0.1")
        .serve("./examples/static")
        .cache_serve_dir()
        .port(8080)
        .start()
        .unwrap();
}

/*- Api endpoints -*/
fn manual_serve(stream: &mut Stream) -> () {
    /*- Respond with the html file -*/
    stream.respond_file(200u16, "examples/static/manual_serve.html");
}
