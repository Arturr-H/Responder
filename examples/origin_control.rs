
use responder::prelude::*;
const STR:&'static str = r#"{"wloc__.buf": "59°14'58.34''N, 17°51'20.00''E"}"#;

fn main() -> () {
    let routes = &[
        Route::Get("", index)
    ];

    Server::new().address("0.0.0.0").port(8080).routes(routes).start().unwrap();
}
fn index(stream:&mut Stream) -> () {
    stream.respond(200, Respond::new().json(STR))
}

