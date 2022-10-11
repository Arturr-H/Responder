// Go to localhost:8080/url_params/put_something_here/and_something_here to see the result

/*- Imports -*/
use std::{net::TcpStream, collections::HashMap};
use responder::{*, request::info::Method, response::Respond, response::respond};

/*- Initialize -*/
fn main() {

    /*- Initiaize routes -*/
    let routes = Route::Stack("", &[
        Route::Stack("url_params", &[
            Route::Stack(":param_1:", &[
                // We'll use 'Function::SP' because it takes parameters as a function param
                Route::Tail(Method::GET, ":some_other_param:", Function::SP(api_endpoint_with_url_params))
            ]),
        ])
    ]);

    /*- Initiaize server -*/
    ServerConfig::new()
        .routes(routes)
        .address("127.0.0.1")
        .port(8080)
        .start()
        .unwrap();
}

fn api_endpoint_with_url_params(mut stream:&mut TcpStream, params:&HashMap<&str, &str>) -> () {
    respond(
        &mut stream,
        200u16,
        Respond::text(
            &format!(
                "{:?}",

                // Params is a hashmap, just send all keys and values in it
                &params
            )
        )
    );
}