/*- Imports -*/
use std::{ net::TcpStream, io::Write };
use crate::response::{ STATUS_CODES, Respond, ResponseType, ImageType };

/*- Structs, enums & unions -*/
/// A simple wrapper for the TcpStream struct, which we want because
/// it eliminates the need of importing more libs from std. This will
/// also be a way of implementing functionality for requests like respond()
/// in a more simpler fashion.
pub struct Stream {
    stream_inner: TcpStream,
}

/*- Method implementations -*/
impl Stream {
    /// Repond quickly using this function
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
    pub fn respond(&mut self, status:u16, respond:Option<Respond>) {

        /*- Get the status string -*/
        let status_msg = STATUS_CODES.iter().find(|&x| x.0 == &status).unwrap_or(&(&0u16, "Internal error - Missing status code")).1;

        /*- Get the response type -*/
        let mut response_type:&str = "text/plain";
        match &respond {
            Some(r) => {
                response_type = match &r.response_type {
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
                };
            },
            None => ()
        };

        /*- If content was provided -*/
        if let Some(content) = respond {
            /*- Grab additional headers -*/
            let additional_headers = match content.additional_headers {
                Some(headers) => vec!["\r\n", &headers.join("\r\n")].join(""),
                None => String::new()
            };

            /*- Write the status & content to the stream -*/
            if self.stream_inner.write(
                format!(
                    "HTTP/1.1 {}\r\nContent-Length: {}\r\nContent-Type: {}\r\n{}\r\n{}",
                    status, content.content.len(), response_type, additional_headers, content.content
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
            Some(Respond {
                response_type: crate::response::ResponseType::Html,
                content: format!(
                    "<html><head><meta http-equiv=\"refresh\" content=\"0; url={}\" /></head><body><a href=\"{}\">Click here if you are not redirected</a></body></html>",
                    url,
                    url
                ),
                additional_headers: Some(vec![format!("Location: {}", url)]),
            })
        );
    }
}

/*- Conversions -*/
impl From<TcpStream> for Stream {

    /// Convert TcpStream into Stream struct.
    fn from(stream_inner: TcpStream) -> Self {
        Self { stream_inner }
    }
}

