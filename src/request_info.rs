use std::fmt;

/*- Structs, enums & unions -*/
/// A struct containing valuable information about a
/// http-request, like the method, path and version
#[derive(Clone, Copy)]
pub struct RequestInfo<'lf> {
    pub method: Method,
    pub path:   &'lf str,
    pub version:&'lf str,
}

/// A struct containing all http-methods
/// that are supported by the server
#[derive(Debug, Clone, Copy)]
pub enum Method { GET, POST, PUT, DELETE, HEAD, OPTIONS, CONNECT, TRACE, PATCH, UNKNOWN }

/*- Method implementations -*/
impl RequestInfo<'_> {

    /// Parses the request string into valuable information,
    /// like the http-method, path and version
    /// 
    /// Example:
    /// ```
    /// let info:RequestInfo = RequestInfo::parse_req(&request);
    /// ```
    pub fn parse_req(request:&str) -> Result<RequestInfo, ()> {
        /*- Get the lines -*/
        let mut lines = request.split::<&str>("\r\n");

        /*- The first line contains info about the request -*/
        let line = match Iterator::nth(&mut lines, 0) {
            Some(e) => e,
            None => {
                return Err(())
            }
        };

        /*- First lines look something like this: "GET / HTTP/1.1\r\n" -*/
        let info_str = line.split_whitespace().collect::<Vec<&str>>();
        let (method, path, version):(&str, &str, &str) = (
            info_str.get(0).unwrap_or(&""),
            info_str.get(1).unwrap_or(&""),
            info_str.get(2).unwrap_or(&""),
        );

        /*- Parse the method -*/
        let method = match method {
            "GET" => Method::GET,
            "POST" => Method::POST,
            "PUT" => Method::PUT,
            "DELETE" => Method::DELETE,
            "HEAD" => Method::HEAD,
            "OPTIONS" => Method::OPTIONS,
            "CONNECT" => Method::CONNECT,
            "TRACE" => Method::TRACE,
            "PATCH" => Method::PATCH,
            _ => Method::UNKNOWN,
        };

        /*- Return -*/
        Ok(RequestInfo { method, path, version })
    }
}

impl std::cmp::PartialEq for Method {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Method::GET, Method::GET) => true,
            (Method::POST, Method::POST) => true,
            (Method::PUT, Method::PUT) => true,
            (Method::DELETE, Method::DELETE) => true,
            (Method::HEAD, Method::HEAD) => true,
            (Method::OPTIONS, Method::OPTIONS) => true,
            (Method::CONNECT, Method::CONNECT) => true,
            (Method::TRACE, Method::TRACE) => true,
            (Method::PATCH, Method::PATCH) => true,
            _ => false,
        }
    }
}

impl fmt::Debug for RequestInfo<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RequestInfo (m: {:?}, p: {}, v: {})", self.method, self.path, self.version)
    }
}