/*- Global allowings -*/
#![allow(
    dead_code
)]

/*- Imports -*/
use crate::{ respond, Respond };
use std::{
    net::TcpStream, collections::HashMap
};


/*- Constants -*/


/*- Structs, enums & unions -*/


/*- Functions -*/

/// Require that all requests to an endpoint must contain specific headers.
/// Will return a bool that indicates wether headers were missing or not.
/// If this function returns true, the server will automatically respond.
/// 
/// ## Examples
/// ```
/// fn api_endpoint(mut stream:TcpStream, headers:HashMap<&str, &str>) -> () {
///     /* Require the "authorization" header to be set.
///         Return if headers didn't exist              */
///     if require_headers(&mut stream, headers, &["authorization"]) { return; };
/// 
///     respond(&mut stream, 200u16, Respond::text("All headers exist!"));
/// }
/// ```
fn require_headers(
    mut stream:TcpStream,
    headers:HashMap<&str, &str>,
    required:&[&str]
) -> bool {
    /*- Get all headers -*/
    let keys:Vec<&&str> = headers.keys().collect();

    /*- Create vec with capacity so that we won't
        need to allocate everytime we update the vec -*/
    let mut missing_headers:Vec<&&str> = Vec::with_capacity(required.len());

    /*- Iterate over all headers -*/
    for key in required {
        /*- Check if headers do not contain the current required header -*/
        if !keys.contains(&key) {
            missing_headers.push(&key);
        };
    };

    /*- Check if anything was missing -*/
    if !missing_headers.is_empty() {
        respond(
            &mut stream,
            400u16,
            Respond::text(
                &format!(
                    "Missing headers: [{:?}]",
                    missing_headers
                )
            )
        );

        /*- Return -*/
        true
    }

    /*- Return that we didn't write to stream -*/
    else { false }
}