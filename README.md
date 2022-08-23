## Rust web framework

Easy to use, easy to set up.
```rust

use rust_web_framework::*;

/*- Initialize -*/
fn main() {

    /*- Initiaize routes -*/
    let routes = Route::Stack("", &[
        Route::Stack("t1", &[
            Route::Tail(Method::GET, "value", Function::S(|stream| {
                std::thread::sleep(std::time::Duration::from_secs(10));
                respond(stream, 200u16, Some(Respond {
                    response_type: ResponseType::Text,
                    content: "Hi".to_string()
                }));
            })),
            Route::Stack("t2", &[
                Route::Tail(Method::POST, "value", Function::S(|_| {println!("NOt should")})),
                Route::Tail(Method::GET, "value", Function::S(|_| {println!("Yes should")})),
                Route::Stack("t3", &[
                    Route::Tail(Method::GET, "value", Function::S(|_| {println!("awod")})),
                ]),
            ]),
        ]),
    ]);

    /*- Initiaize server -*/
    start(ServerConfig {
        addr: "127.0.0.1",
        port: 8080u16,
        serve: Some("./static"),
        not_found: Some("./static/404.html"),
        routes,
    }).unwrap();
}
```