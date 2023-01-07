//! This is a small mockup server intended for testing. Will be started using docker
use responder::prelude::*;

fn main() -> () {
    let routes = &[
        Route::Get("url-params/:param1:/:param2:", url_params),
        Route::Get("headers", headers),
        Route::Get("ok", payload_ok),
        Route::Get("file", file),
        Route::Get("redirect", redirect),
        Route::Get("body", body)
    ];
    
    Server::new()
        .port(6102)
        .address("0.0.0.0")
        .routes(routes)
        .start()
        .unwrap();
}

/* Respond with OK JSON payload */
fn payload_ok(stream:&mut Stream) -> () {
    stream.payload_status(200);
}

/* Will return with URL-params in JSON array */
fn url_params(stream:&mut Stream) -> () {
    stream.respond(200, Respond::new().json(
        &format!(
            "{{\"args\":{:?}}}",
            stream.params.values()
        )
    ));
}

/* Respond with headers which were provided in JSON array */
fn headers(stream:&mut Stream) -> () {
    stream.respond(200, Respond::new().json(
        &format!(
            "{{\"args\":{:?}}}",
            stream.headers
        )
    ));
}

/* Respond with request body */
fn body(stream:&mut Stream) -> () {
    stream.respond(
        200,
        Respond::new().text(
            &stream.body
        )
    );
}

/* Redirect to 200-ok payload endpoint */
fn redirect(stream:&mut Stream) -> () {
    stream.redirect("/ok");
}

/* Respond with file */
fn file(stream:&mut Stream) -> () {
    stream.respond_file(200, "/path/to/file");
}
