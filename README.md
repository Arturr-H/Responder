<p align="center" style="width:100%">
    <img src="https://img.shields.io/github/repo-size/Arturr-H/Responder">
    <img src="https://img.shields.io/crates/v/Responder">
    <img src="https://img.shields.io/crates/d/Responder">
    <img src="https://img.shields.io/crates/l/Responder">
    <img src="https://img.shields.io/docsrs/responder">
    <img src="https://img.shields.io/github/actions/workflow/status/Arturr-H/Responder/rust.yml">
</p>

# Responder

Easy to use, easy to set up. Here's an example of a simple web-server:
```rust
use responder::prelude::*;

fn main() {

    /*- Initiaize routes -*/
    let routes = &[
        Route::Stack("path", &[
            Route::Get("endpoint", endpoint),

            Route::Stack("subpath", &[
                // ...
            ]),
        ]),
    ];

    /*- Initiaize server -*/
    Server::new()
        // This will be localhost, use 
        // 0.0.0.0 if using e.g. docker
        .address("127.0.0.1") 
        .port(8080)

        // Serve static files from a folder
        .serve("./static")
        .routes(routes)
        .start()
        .unwrap();

    // Go to 'localhost:8080/path/enpoint' to see results
}

/* Will respond with the http-status code 200 */
fn endpoint(stream:&mut Stream) -> () {
    stream.respond_status(200);
}
```

Simple, isn't it? *Now where and how do I handle all my requests?*
<br /><br />
The `Stream` and `Respond` structs help you manage incoming requests as well as providing you many options for building http-responses.
<br />
### › Stream 🌊
The `Stream` struct is passed as a parameter to every endpoint-function. It contains valuable information, together with salient methods for your needs. Here's an exaple of an endpoint function utilizing the features of the `Stream` struct:

```rust
/* Will respond with the http-status code 200 */
fn endpoint(stream:&mut Stream) -> () {
    stream.respond_status(200);
}

/* Will respond with secret JSON data if request */
fn endpoint_2(stream:&mut Stream) -> () {
    /* Request data */
    let headers    = &stream.headers;
    let cookies    = &stream.get_cookies();
    let url_params = &stream.params;
    let body       = &stream.body;

    /* Check if request header "pin_code" is correct */
    if let Some(pin_code) = headers.get("pin_code") {
        if pin_code != &"12345" {
            /* Unauthorized (Invalid pin-code) */
            return stream.respond_status(401)
        };
    }else {
        /* Bad request (missing header) */
        return stream.respond_status(40) 
    }

    /* Respond */
    stream.respond(
        200,
        Respond::new()
            /* TIP: use the json!() macro from the
                `serde_json` crate instead of strings */
            .json("{{\"secret-password\": \"password123\"}}")
    );
}
```

It's wrapped around the `TcpStream` struct from `std::net`. Therefore there are many more ways of handling requests. The `TcpStream` can be aquired by calling `stream.get_mut_inner_ref()`.

### › Respond 🌍
The `Respond` struct is used to construct HTTP responses. It's mostly constructed using the "*builder pattern*". Here's one example of how it could be used:

```rust
/* Will respond with some text */
fn endpoint(stream:&mut Stream) -> () {
    stream.respond(
        200,
        Respond::new()
            .text("Hello, world!")
    );
}
```

## Security 🚨
Now that we've covered the basics of `responder`, we'll shortly dig into the security. Rust, by default is secure. Therefore we don't need to be worried about memory leaks and more. However, that won't stop people from getting access to restricted endpoints. `responder` has a solution for that. It's called `origin-control`. It's an enum variant in the `Route` struct named `ControlledStack`, and it's main purpose is to check wether the incoming request meets some criteria, and then either ditch the request, or grant it access to the inner endpoints. Here's an example of how you could do that:

```rust
let routes = &[
    /* Will be accessible to all requests */
    Route::Get("non-admin", non_admin_page),

    /* Everything inside `Route::ControlledStack`
        will be accessible to all requests matching
        the `origin_control` functions criteria */
    Route::ControlledStack(origin_control, "admin", &[
        Route::Get("secret-data", secret_data),
    ])
];

/* Create the origin control function */
fn origin_control_function(stream:&mut Stream) -> bool {
    /* Check if request has the correct token */
    if let Some(token) = stream.headers.get("token") {
        if token == &"password123" {
            /* Return true indicating that
                the request matches criteria */
            return true
        };
    };

    /* Return false indicating that the
        request does not match criteria */
    false
}
```

## Contribution 🛠️
I'm only 16 years old, and I do not have a lot of knowledge regarding backend security / making projects in general. Therefore contributions and feedback would be very helpful and appreciated!<br /><br />
There are many ways to contribute, either make a push request, create an issue, mail me some things to do / feedback (`artur.h.0427@gmail.com`), or use the crate in your own projects!
