/*- Imports -*/
use std::{ net::TcpStream, io::Write, collections::HashMap, hash::Hash };
use crate::response::{ STATUS_CODES, Respond, ResponseType, ImageType };

/*- Structs, enums & unions -*/
/// A simple wrapper for the TcpStream struct, which we want because
/// it eliminates the need of importing more libs from std. This will
/// also be a way of implementing functionality for requests like respond()
/// in a more simpler fashion.
/// 
/// Also contains request information like body, params and headers
pub struct Stream<'lf> {
    /// We won't take a mutable reference of TcpStream because we want
    /// full ownership of it which will give us mutable access to it anyways.
    stream_inner: TcpStream,

    /// If stream_inner has aleady been written to (Should only be written to once)
    buf_written_to: bool,

    /// Body is only used in POST requests. Often used for sending & recieving 
    /// big chunks of data like images or files.
    pub body: String,

    /// URL-parameters which will be set in routes by using :_: in tail
    pub params: HashMap<String, String>,

    /// Header keys and values which will specified in fetch requests
    pub headers: HashMap<&'lf str, &'lf str>
}

/*- Method implementations -*/
impl<'a> Stream<'a> {
    /// Respond quickly using this function
    /// ## Example
    /// ```
    /// /* Repond with 200 OK */
    /// stream.respond(200u16, None);
    /// 
    /// /* Repond with text */
    /// stream.respond(200u16, Some(Respond {
    ///     content: String::from("Hello world!"),
    ///     response_type: ResponseType::Text,
    /// }));
    /// 
    /// /* Repond with JSON */
    /// stream.respond(200u16, Some(Respond {
    ///     /* Better to use a library like serde
    ///        to convert structs to JSON strings */
    ///     content: String::from("{\"key\":\"value\"}"),
    ///     response_type: ResponseType::Text,
    /// }));
    /// ```
    pub fn respond(&mut self, status:u16, respond:Respond) {
        /*- Check buffer write access -*/
        if self.buf_written_to { return; };
        self.buf_written_to = true;

        /*- Get the status string -*/
        let status_msg = STATUS_CODES.iter().find(|&x| x.0 == &status).unwrap_or(&(&status, "Internal error - Missing status code")).1;

        /*- Get the response type -*/
        let mut response_type:&str = match respond.response_type {
            ResponseType::Text => "text/plain",
            ResponseType::Json => "application/json",
            ResponseType::Html => "text/html",
            ResponseType::Image(c)  => {
                match c {
                    ImageType::Jpeg => "image/jpeg",
                    ImageType::Png => "image/png",
                    ImageType::Gif => "image/gif",
                    ImageType::Webp => "image/webp",
                    ImageType::Svg => "image/svg+xml",
                }
            },
            ResponseType::Custom(custom) => &custom
        };

        /*- If content was provided -*/
        if let Some(content) = respond.content {
            /*- Grab additional headers -*/
            let additional_headers = match respond.additional_headers {
                Some(headers) => vec!["\r\n", &headers.join("\r\n")].join(""),
                None => String::new()
            };

            /*- Write the status & content to the stream -*/
            if self.stream_inner.write(
                format!(
                    "HTTP/1.1 {}\r\nContent-Length: {}\r\nContent-Type: {}\r\n{}\r\n{}",
                    status, content.len(), response_type, additional_headers, content
                ).as_bytes()
            ).is_ok() { };
        }else {
            /*- Write the status to the stream -*/
            if self.stream_inner.write(
                format!(
                    "HTTP/1.1 {}\r\n\r\n{} {}",
                    status, status, status_msg
                ).as_bytes()
            ).is_ok() { };
        };

        /*- Flush the stream -*/
        self.stream_inner.flush().ok();
    }

    /// Respond with just status code
    /// ## Example
    /// ```
    /// /* Repond with 200 OK */
    /// stream.respond_status(200u16);
    /// ```
    pub fn respond_status(&mut self, status:u16) {
        /*- Check buffer write access -*/
        if self.buf_written_to { return; };
        self.buf_written_to = true;

        /*- Get the status string -*/
        let status_msg = STATUS_CODES.iter().find(|&x| x.0 == &status).unwrap_or(&(&status, "Internal error - Missing status code")).1;

        /*- Get the response type -*/
        let mut response_type:&str = "text/plain";

        /*- Write the status to the stream -*/
        if self.stream_inner.write(
            format!(
                "HTTP/1.1 {}\r\n\r\n{} {}",
                status, status, status_msg
            ).as_bytes()
        ).is_ok() { };

        /*- Flush the stream -*/
        self.stream_inner.flush().ok();
    }

    /// Get a mutable reference of the inner stream because
    /// the stream_inner key isn't exposed publicly.
    /// 
    /// ## Examples
    /// ```
    /// stream.get_mut_inner_ref(); // -> &mut TcpStream
    /// ```
    pub fn get_mut_inner_ref(&mut self) -> &mut TcpStream {
        &mut self.stream_inner
    }

    /// Redirect requests to url, might not work with all browsers so
    /// a link will appear which users can click incase it doesn't work.
    /// 
    /// ## Examples
    /// ```
    /// fn redirect_user(stream:&mut Stream) -> () {
    ///     stream.redirect("https://google.com");
    /// }
    /// ```
    pub fn redirect(&mut self, url:&str) -> () {
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
    pub fn set_body(&mut self, body:String) ->                          &mut Self { self.body = body; self }
    pub fn set_headers(&mut self, headers:HashMap<&'a str, &'a str>) -> &mut Self { self.headers = headers; self }
    pub fn set_params(&mut self, params:HashMap<String, String>) ->     &mut Self { self.params = params; self }

    /// Require headers to be specified. If they are not, this
    /// function will repsond with an array containing missing
    /// headers. Return true indicating that the request should
    /// be cancelled or not. 
    /// 
    /// ## Examples
    /// ```
    /// /*- Return if headers were not specified -*/
    /// if stream.expect_headers(&["authentification"], true) { return; };
    /// ```
    pub fn expect_headers(&mut self, headers:&[&str], ignore_caps:bool) -> bool {
        let request_headers:Vec<&&str> = self.headers
                .keys()
                .collect();

        for expected_header in headers {
            if !request_headers.contains(&expected_header) {
                self.respond(
                    400u16,
                    Respond::new().text(
                        &format!("This endpoint requires these headers: {headers:?}")
                    )
                );
                return true;
            }
        };

        false
    }

    /// Require headers to be specified, but ignore encapsulation. If
    /// headers are not set, this function will repsond with an array
    /// containing missing headers. Return true indicating that the
    /// request should be cancelled or not.
    /// 
    /// ## Examples
    /// ```
    /// /*- Return if headers were not specified -*/
    /// if stream.expect_headers(&["authentification"], true) { return; };
    /// ```
    pub fn expect_headers_ignore_caps(&mut self, headers:&[&str]) -> bool {
        let request_headers:Vec<String> = self.headers
                .keys()
                .collect::<Vec<&&str>>()
                .iter()
                .map(|e| e.to_ascii_lowercase())
                .collect();

        for expected_header in headers {
            if !request_headers.contains(&expected_header.to_ascii_lowercase()) {
                self.respond(
                    400u16,
                    Respond::new().text(
                        &format!("This endpoint requires these headers: {headers:?}")
                    ));
                return true;
            }
        };

        false
    }
}

/*- Conversions -*/
impl<'a> From<TcpStream> for Stream<'a> {

    /// Convert TcpStream into Stream struct.
    fn from(stream_inner: TcpStream) -> Self {
        Self {
            stream_inner,
            buf_written_to: false,
            body: String::new(),
            params: HashMap::new(),
            headers: HashMap::new()
        }
    }
}
