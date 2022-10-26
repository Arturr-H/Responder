## Responder

Easy to use, easy to set up.
```rust
use responder::*;

/*- Initialize -*/
fn main() {

    /*- Initiaize routes -*/
    let routes = Route::Stack("", &[
        Route::Stack("path", &[
            Route::Get("enpoint", some_function),
            Route::Get("enpoint2", some_other_function),
        ]),
    ]);

    /*- Initiaize server -*/
    Server::new()
        .address("127.0.0.1")            // This will be localhost, use 0.0.0.0 if using docker
        .port(8080)
        .serve("./static")              // Serve static files from a folder
        .not_found("./static/404.html") // Where to direct users going to a path which doesn't exist
        .threads(8)                     // How many threads to handle all requests
        .routes(routes)
        .start()
        .unwrap();

    // Go to 'localhost:8080/path/enpoint' to see results
}
```