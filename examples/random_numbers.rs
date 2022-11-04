// Go to localhost:8080 to see the result

/*- Imports -*/
use responder::prelude::*;
use rand::Rng;

/*- Initialize -*/
fn main() {

    /*- Initiaize routes -*/
    let routes = &[
        Route::Get("", respond_with_random_number),
    ];

    /*- Initiaize server -*/
    Server::new()
        .routes(routes)
        .address("127.0.0.1")
        .port(8080)
        .start()
        .unwrap();
}

/*- Api endpoints -*/
fn respond_with_random_number(stream:&mut Stream) -> () {
    /*- Get random number -*/
    let random_number = rand::thread_rng().gen_range(0..100);

    /*- Respond with the random number -*/
    stream.respond(
        200u16,
        Respond::new().text(
            &format!(
                "Random number between 0 and 100: {}",
                &random_number
            )
        )
    );
}
