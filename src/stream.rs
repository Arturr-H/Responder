/*- Imports -*/
use crate::{
    response::{ImageType, Respond, ResponseType, STATUS_CODES},
    FILE_CACHE,
};
use std::{
    collections::HashMap,
    fs::File,
    hash::Hash,
    io::Read,
    io::Write,
    net::TcpStream,
    path::{Path, PathBuf},
};

/*- TEMP Cors -*/
const CORS:&'static str = "\r\nAccess-Control-Allow-Origin: *\r\nAccess-Control-Allow-Headers: *\r\nAccess-Control-Allow-Methods: GET, POST, PUT, DELETE, OPTIONS, HEAD\r\nAccess-Control-Max-Age: 86400";

/*- Structs, enums & unions -*/
/// A simple wrapper for the TcpStream struct, which we want because
/// it eliminates the need of importing more libs from std. This will
/// also be a way of implementing methods for requests like respond()
/// in a more simpler fashion.
///
/// Also contains request information such as body, params and headers and more
pub struct Stream<'lf> {
    /// We won't take a mutable reference of TcpStream because we want
    /// full ownership of it which will give us mutable access to it anyways.
    stream_inner: TcpStream,

    /// If stream_inner has aleady been written to (Should only be written to once)
    buf_written_to: bool,

    /// Body is only used in POST requests. Often used for sending & recieving
    /// big chunks of data like images or files.
    body: String,

    /// URL-parameters which will be set in routes by using :_: in tail
    params: HashMap<String, String>,

    /// Header keys and values which will specified in fetch requests
    headers: HashMap<&'lf str, &'lf str>,

    /// Cors
    cors: bool,
}

/*- Method implementations -*/
impl<'a> Stream<'a> {
    /// Send back an http-response
    /// ## Example
    /// ```
    /// use responder::prelude::*;
    ///
    /// /* Repond with text */
    /// fn endpoint_text(stream:&mut Stream) -> () {
    ///     stream.respond(200u16, Respond::new().text("Hello, world!"));
    /// }
    ///
    /// /* Repond with json */
    /// fn endpoint_json(stream:&mut Stream) -> () {
    ///     stream.respond(200u16, Respond::new().json("{{\"key\":\"value\"}}"));
    /// }
    /// ```
    pub fn respond(&mut self, status: u16, respond: Respond) {
        /*- Check buffer write access -*/
        if self.buf_written_to {
            return;
        };
        self.buf_written_to = true;

        /*- Get the status string -*/
        let status_msg = STATUS_CODES
            .iter()
            .find(|&x| x.0 == &status)
            .unwrap_or(&(&status, "Internal error - Missing status code"))
            .1;

        /*- Get the response type -*/
        let mut response_type: &str = match respond.response_type {
            ResponseType::Json => "application/json",
            ResponseType::Js => "text/javascript",
            ResponseType::Text => "text/plain",
            ResponseType::Html => "text/html",
            ResponseType::Css => "text/css",
            ResponseType::Image(c) => match c {
                ImageType::Jpeg => "image/jpeg",
                ImageType::Png => "image/png",
                ImageType::Gif => "image/gif",
                ImageType::Webp => "image/webp",
                ImageType::Svg => "image/svg+xml",
            },
            ResponseType::Custom(custom) => &custom,
        };
        let cors = if self.cors { CORS } else { "" };

        /*- If content was provided -*/
        if let Some(content) = respond.content {
            /*- Grab additional headers -*/
            let additional_headers = match respond.additional_headers {
                Some(headers) => vec!["\r\n", &headers.join("\r\n")].join(""),
                None => String::new(),
            };

            /*- Write the status & content to the stream -*/
            if self.stream_inner.write(
                format!(
                    "HTTP/1.1 {}\r\nContent-Length: {}\r\nContent-Type: {}{additional_headers}{cors}\r\n\r\n{content}",
                    status, content.len(), response_type
                ).as_bytes()
            ).is_ok() { };
        } else {
            /*- Write the status to the stream -*/
            if self
                .stream_inner
                .write(
                    format!("HTTP/1.1 {}{cors}\r\n\r\n{} {}", status, status, status_msg)
                        .as_bytes(),
                )
                .is_ok()
            {};
        };

        /*- Flush the stream -*/
        self.stream_inner.flush().ok();
    }

    /// Respond with just status code
    /// ## Example
    /// ```
    /// use responder::prelude::*;
    ///
    /// /* Repond with 200 (OK) */
    /// fn endpoint_200(stream:&mut Stream) -> () {
    ///     stream.respond_status(200u16); // Response body will look like this: "200 OK"
    /// }
    ///
    /// /* Repond with 600 (status code doesn't exist) */
    /// fn endpoint_600(stream:&mut Stream) -> () {
    ///     stream.respond_status(600u16); // Response body will look like this: "600 Internal error - Missing status code"
    /// }
    /// ```
    pub fn respond_status(&mut self, status: u16) {
        /*- Check buffer write access -*/
        if self.buf_written_to {
            return;
        };
        self.buf_written_to = true;

        /*- Get the status string -*/
        let status_msg = STATUS_CODES
            .iter()
            .find(|&x| x.0 == &status)
            .unwrap_or(&(&status, "Internal error - Missing status code"))
            .1;
        let cors = if self.cors { CORS } else { "" };

        /*- Get the response type -*/
        let mut response_type: &str = "text/plain";

        /*- Write the status to the stream -*/
        if self
            .stream_inner
            .write(format!("HTTP/1.1 {}{cors}\r\n\r\n{} {}", status, status, status_msg).as_bytes())
            .is_ok()
        {};

        /*- Flush the stream -*/
        self.stream_inner.flush().ok();
    }

    /// Respond with JSON payload, takes `payload` as param, which can be any type that can be converted into a `String`
    /// ## Example
    /// ```
    /// use responder::prelude::*;
    ///
    /// /* Repond with some JSON values (TIP: use the `json!()` macro from the `serde_json` crate) */
    /// fn endpoint_payload(stream:&mut Stream) -> () {
    ///     stream.payload("{{\"key\": \"value\"}}");
    /// }
    /// ```
    pub fn payload(&mut self, payload: impl ToString) {
        self.respond(200, Respond::new().json(&payload.to_string()));
    }

    /// Respond with JSON payload status
    /// ## Example
    /// ```
    /// use responder::prelude::*;
    ///
    /// /* Repond with payload status (real http status will be 200) */
    /// fn endpoint(stream:&mut Stream) -> () {
    ///     stream.payload_status(200);
    /// }
    /// ```
    ///
    /// ## Usecases
    /// In JavaScript, you often want payload status responses instead of http-status responses, for example
    /// you have a backend API with diffrent user-profiles. And on the client-side some user searches for a
    /// non-existing user. We'd like to respond with a payload status of 404, not http-status 404 because that
    /// would make JavaScript think that the route specified in the fetch request is invalid, eventhough it's
    /// valid - only that the user wasn't found.
    pub fn payload_status(&mut self, status: u16) {
        self.payload(format!("{{\"status\":{}}}", status));
    }

    /// Get a mutable reference of the inner stream because
    /// the stream_inner key isn't exposed publicly. Gives
    /// access to more in-depth functionality
    ///
    /// ## Examples
    /// ```
    /// use responder::prelude::*;
    ///
    /// fn endpoint(stream:&mut Stream) -> () {
    ///     stream.get_mut_inner_ref(); // -> &mut TcpStream
    /// }
    /// ```
    pub fn get_mut_inner_ref(&mut self) -> &mut TcpStream {
        &mut self.stream_inner
    }

    /// Redirect requests to url, might not work with all browsers so
    /// a link will appear which users can click incase it doesn't work.
    ///
    /// ## Examples
    /// ```
    /// use responder::prelude::*;
    ///
    /// fn redirect_user(stream:&mut Stream) -> () {
    ///     stream.redirect("https://google.com");
    /// }
    /// ```
    pub fn redirect(&mut self, url: &str) -> () {
        self.respond(
            308u16,
            Respond::new()
                .html(
                    &format!(
                        "<html><head><meta http-equiv=\"refresh\" content=\"0; url={}\" /></head><body><a href=\"{}\">Click here if you are not redirected</a></body></html>",
                        url,
                        url
                    )
                )
                .headers(vec![format!("Location: {}", url)])
        );
    }

    /*- Append request data (body, headers, url-params) to self -*/
    /// Builder pattern for making the `Stream` struct. Not meant to be used
    pub fn set_body(&mut self, body: String) -> &mut Self {
        self.body = body;
        self
    }
    /// Builder pattern for making the `Stream` struct. Not meant to be used
    pub fn set_headers(&mut self, headers: HashMap<&'a str, &'a str>) -> &mut Self {
        self.headers = headers;
        self
    }
    /// Builder pattern for making the `Stream` struct. Not meant to be used
    pub fn set_params(&mut self, params: HashMap<String, String>) -> &mut Self {
        self.params = params;
        self
    }

    /// Require headers to be specified. If they are not, this
    /// function will repsond with an array containing missing
    /// headers. Return true indicating that the request should
    /// be cancelled or not.
    ///
    /// ## Examples
    /// ```
    /// use responder::prelude::*;
    ///
    /// /*- Return if headers were not specified -*/
    /// fn endpoint(stream:&mut Stream) -> () {
    ///     if stream.expect_headers(&["authentification"]) { return; };
    ///
    ///     /* Handle authorized requests... */
    /// }
    /// ```
    pub fn expect_headers(&mut self, headers: &[&str]) -> bool {
        let request_headers: Vec<&&str> = self.headers().keys().collect();

        for expected_header in headers {
            if !request_headers.contains(&expected_header) {
                self.respond(
                    400u16,
                    Respond::new().text(&format!(
                        "This endpoint requires these headers: {headers:?}"
                    )),
                );
                return true;
            }
        }

        false
    }

    /// Require headers to be specified, but ignore encapsulation. If
    /// headers are not set, this function will repsond with an array
    /// containing missing headers. Return true indicating that the
    /// request should be cancelled or not.
    ///
    /// ## Examples
    /// ```
    /// use responder::prelude::*;
    ///
    /// /*- Return if headers were not specified -*/
    /// fn endpoint(stream:&mut Stream) -> () {
    ///     if stream.expect_headers(&["authentification"]) { return; };
    ///
    ///     /* Handle authorized requests... */
    /// }
    /// ```
    pub fn expect_headers_ignore_caps(&mut self, headers: &[&str]) -> bool {
        let request_headers: Vec<String> = self
            .headers
            .keys()
            .collect::<Vec<&&str>>()
            .iter()
            .map(|e| e.to_ascii_lowercase())
            .collect();

        for expected_header in headers {
            if !request_headers.contains(&expected_header.to_ascii_lowercase()) {
                self.respond(
                    400u16,
                    Respond::new().text(&format!(
                        "This endpoint requires these headers: {headers:?}"
                    )),
                );
                return true;
            }
        }

        false
    }

    /*- Respond with file -*/
    /// ## Example
    /// ```
    /// use responder::prelude::*;
    ///
    /// fn endpoint(stream:&mut Stream) -> () {
    ///     stream.respond_file(200u16, "/path/to/file.png")
    /// }
    /// ```
    pub fn respond_file(&mut self, status: u16, path: &str) -> () {
        /*- Grab the path -*/
        let _path = Path::new(path);

        /*- Find if exists in file cache -*/
        if let Ok(fc) = FILE_CACHE.lock() {
            match fc.get(
                &_path
                    .canonicalize()
                    .unwrap_or(PathBuf::from(""))
                    .display()
                    .to_string(),
            ) {
                Some(buf) => {
                    return self.respond(
                        status,
                        Respond::new()
                            .content(&String::from_utf8_lossy(&buf), ResponseType::guess(_path)),
                    )
                }
                None => (),
            }
        };

        /*- Open file -*/
        let content: String = match File::open(_path) {
            Ok(mut e) => {
                let mut content: String = String::new();
                if e.read_to_string(&mut content).is_ok() {}

                content
            }
            Err(_) => String::new(),
        };

        /*- Return -*/
        self.respond(
            status,
            Respond::new().content(&content, ResponseType::guess(_path)),
        )
    }

    /*- Get cookies -*/
    /// ## Example
    /// ```
    /// use responder::prelude::*;
    /// use std::collections::HashMap;
    ///
    /// fn endpoint(stream:&mut Stream) -> () {
    ///     let cookies:HashMap<&str, &str> = stream.get_cookies();
    ///
    ///     /* Try get some cookie */
    ///     let token = cookies.get("token").unwrap();
    /// }
    /// ```
    pub fn get_cookies(&self) -> HashMap<&str, &str> {
        let mut cookies: HashMap<&str, &str> = HashMap::new();

        if let Some(cookie) = self.headers.get("Cookie") {
            for cookie in cookie.split("; ") {
                let cookie: Vec<&str> = cookie.split("=").collect();
                cookies.insert(
                    match cookie.get(0) {
                        Some(e) => e,
                        None => continue,
                    },
                    match cookie.get(1) {
                        Some(e) => e,
                        None => continue,
                    },
                );
            }
        };

        cookies
    }

    /*- Getters -*/
    /// Aquire a reference to the body string
    pub fn body(&self) -> &String {
        &self.body
    }

    /// Aquire a reference to the URL-parameters
    pub fn params(&self) -> &HashMap<String, String> {
        &self.params
    }

    /// Aquire a reference to the request headers
    pub fn headers(&self) -> &HashMap<&str, &str> {
        &self.headers
    }

    /// Should not be used. Is only used internally during startup
    pub fn enable_cors(&mut self) -> () {
        self.cors = true
    }
}

/*- Conversions -*/
impl<'a> From<TcpStream> for Stream<'a> {
    /// Convert TcpStream into Stream struct.
    fn from(stream_inner: TcpStream) -> Self {
        Self {
            cors: false,
            stream_inner,
            buf_written_to: false,
            body: String::new(),
            params: HashMap::new(),
            headers: HashMap::new(),
        }
    }
}
