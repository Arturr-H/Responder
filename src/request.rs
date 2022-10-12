/*- Global allowings -*/
#![allow(
    dead_code
)]

/*- Imports -*/
use crate::{ respond, Respond };
use std::{
    net::TcpStream, collections::HashMap
};

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
pub fn require_headers(
    stream:&mut TcpStream,
    headers:HashMap<&str, &str>,
    required:&[&str]
) -> bool {
    /*- Get all headers -*/
    let keys = headers.keys();

    /*- Create vec with capacity so that we won't
        need to allocate everytime we update the vec -*/
    let mut missing_headers:Vec<&&str> = Vec::with_capacity(required.len());

    /*- Iterate over all headers -*/
    for key in required {
        /*- Check if headers do not contain the current required header -*/
        if !keys.clone().any(|e| e == key) {
            /*- Add the missing header to the vec -*/
            missing_headers.push(key);
        };
    };

    /*- Check if anything was missing -*/
    if !missing_headers.is_empty() {
        respond(
            stream,
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

/*- Info module -*/
pub mod info {
    use std::fmt;

    /*- Structs, enums & unions -*/
    /// A struct containing valuable information about a
    /// http-request, like the method, path and version
    #[derive(Clone, Copy)]
    pub struct RequestInfo<'lf> {
        /// The http-method
        pub method: Method,

        /// The path to the endpoint
        pub path:   &'lf str,

        /// Http verison
        pub version:&'lf str,
    }

    /// An enum containing all http-methods
    /// that are supported by the server
    #[derive(Debug, Clone, Copy)]
    pub enum Method {

        /// Retrieve data using URI
        GET,

        /// Send data using body
        POST,
        
        /// Replaces all current representations of the target resource with the uploaded content.
        PUT,
        
        /// Removes all current representations of the target resource given by a URI.
        DELETE,
        
        /// Same as GET, but transfers the status line and header section only.
        HEAD,
        
        /// Describes the communication options for the target resource.
        OPTIONS,
        
        /// Establishes a tunnel to the server identified by a given URI.
        CONNECT,
        
        /// Performs a message loop-back test along the path to the target resource.
        TRACE,
        
        /// A PATCH request is considered a set of instructions on how to modify a resource.
        PATCH,
        
        UNKNOWN
    }

    /*- Method implementations -*/
    impl RequestInfo<'_> {

        /// Parses the request string into valuable information,
        /// like the http-method, path and version
        /// 
        /// Example:
        /// ```
        /// let info:RequestInfo = RequestInfo::parse_req(&request);
        /// ```
        pub fn parse_req(request:&str) -> Result<RequestInfo, u8> {
            /*- Get the lines -*/
            let mut lines = request.split::<&str>("\r\n");

            /*- The first line contains info about the request -*/
            let line = match Iterator::nth(&mut lines, 0) {
                Some(e) => e,
                None => {
                    return Err(0)
                }
            };

            /*- First lines look something like this: "GET / HTTP/1.1\r\n" -*/
            let info_str = line.split_whitespace().collect::<Vec<&str>>();
            let (method, path, version):(&str, &str, &str) = (
                info_str.first().unwrap_or(&""),
                info_str.get(1).unwrap_or(&""),
                info_str.get(2).unwrap_or(&""),
            );

            /*- Parse the method -*/
            let method = match method {
                "GET" =>    Method::GET,
                "POST" =>   Method::POST,
                "PUT" =>    Method::PUT,
                "DELETE" => Method::DELETE,
                "HEAD" =>   Method::HEAD,
                "OPTIONS" => Method::OPTIONS,
                "CONNECT" => Method::CONNECT,
                "TRACE" =>  Method::TRACE,
                "PATCH" =>  Method::PATCH,
                _ => Method::UNKNOWN,
            };

            /*- Return -*/
            Ok(RequestInfo { method, path, version })
        }
    }

    impl std::cmp::PartialEq for Method {
        fn eq(&self, other: &Self) -> bool {
            matches!(
                  (self, other),
                  (Method::GET, Method::GET)
                | (Method::POST, Method::POST)
                | (Method::PUT, Method::PUT)
                | (Method::DELETE, Method::DELETE)
                | (Method::HEAD, Method::HEAD)
                | (Method::OPTIONS, Method::OPTIONS)
                | (Method::CONNECT, Method::CONNECT)
                | (Method::TRACE, Method::TRACE)
                | (Method::PATCH, Method::PATCH))
        }
    }

    impl fmt::Debug for RequestInfo<'_> {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "RequestInfo (m: {:?}, p: {}, v: {})", self.method, self.path, self.version)
        }
    }
}
