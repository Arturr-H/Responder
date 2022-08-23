## Rust web framework

Easy to use, easy to set up.
```rust
use rust_web_framework::*;

/*- Initialize -*/
fn main() {

    /*- Initiaize routes -*/
    let routes = Route::Stack("", &[
        Route::Stack("path", &[
            Route::Tail(Method::GET, "enpoint", Function::S(some_function)),
            Route::Tail(Method::GET, "enpoint2", Function::S(some_other_function)),
        ]),
    ]);

    /*- Initiaize server -*/
    start(ServerConfig {
        addr: "127.0.0.1", // This will be localhost
        port: 8080u16,     // Port, use 0.0.0.0 if using docker
        serve: Some("./static"),              // Serve static files from a folder
        not_found: Some("./static/404.html"), // Where to direct users going to a path which doesn't exist
        routes,
    }).unwrap();

    // Go to 'localhost:8080/path/enpoint' to see results
}
```