/*- Imports -*/
use crate::{stream::Stream, Server};
use std::{
    fs,
    io::{Read, Write},
    path::Path,
};

/*- Constants -*/
pub const STATUS_CODES: &[(&u16, &str); 58] = &[
    /* 500 */
    (&500, "Internal Server Error"),
    (&501, "Not Implemented"),
    (&502, "Bad Gateway"),
    (&503, "Service Unavailable"),
    (&504, "Gateway Timeout"),
    (&505, "HTTP Version Not Supported"),
    (&506, "Variant Also Negotiates"),
    (&507, "Insufficient Storage"),
    (&508, "Loop Detected"),
    (&510, "Not Extended"),
    (&511, "Network Authentication Required"),
    /* 400 */
    (&400, "Bad Request"),
    (&401, "Unauthorized"),
    (&402, "Payment Required"),
    (&403, "Forbidden"),
    (&404, "Not Found"),
    (&405, "Method Not Allowed"),
    (&406, "Not Acceptable"),
    (&407, "Proxy Authentication Required"),
    (&408, "Request Timeout"),
    (&409, "Conflict"),
    (&410, "Gone"),
    (&411, "Length Required"),
    (&412, "Precondition Failed"),
    (&413, "Payload Too Large"),
    (&414, "URI Too Long"),
    (&415, "Unsupported Media Type"),
    (&416, "Range Not Satisfiable"),
    (&417, "Expectation Failed"),
    (&418, "I'm a teapot"),
    (&421, "Misdirected Request"),
    (&422, "Unprocessable Entity"),
    (&423, "Locked"),
    (&424, "Failed Dependency"),
    (&425, "Too Early"),
    (&426, "Upgrade Required"),
    (&428, "Precondition Required"),
    (&429, "Too Many Requests"),
    (&431, "Request Header Fields Too Large"),
    (&451, "Unavailable For Legal Reasons"),
    /* 300 */
    (&300, "Multiple Choices"),
    (&301, "Moved Permanently"),
    (&302, "Found"),
    (&303, "See Other"),
    (&304, "Not Modified"),
    (&305, "Use Proxy"),
    (&306, "Switch Proxy"),
    (&307, "Temporary Redirect"),
    (&308, "Permanent Redirect"),
    /* 200 */
    (&200, "OK"),
    (&201, "Created"),
    (&202, "Accepted"),
    (&204, "No Content"),
    (&205, "Reset Content"),
    (&206, "Partial Content"),
    (&207, "Multi-status"),
    (&208, "Already reported"),
    (&226, "IM Used"),
];

/*- Structs, enums & unions -*/
#[derive(Clone, Debug)]
/// The respond struct will mostly be constructed by using the builder
/// pattern. Often found in the stream.respond(_, _); function. Takes
/// `response type`, `content` and `additional_headers` as fields.
///
/// ## Examples
/// ```
/// use responder::prelude::*;
///
/// let text_response = Respond::new().text("Hello, world!");
/// let json_response = Respond::new().json("{{\"key\": \"value\"}}");
/// ```
pub struct Respond {
    pub response_type: ResponseType,
    pub content: Option<String>,
    pub additional_headers: Option<Vec<String>>,
}

#[derive(Clone, Copy, Debug)]
/// What data type the server will respond with
pub enum ResponseType {
    Text,
    Css,
    Json,
    Html,
    Js,
    Image(ImageType),
    Custom(&'static str),
}

/// What type of image server will respond with
#[derive(Clone, Copy, Debug)]
pub enum ImageType {
    Jpeg,
    Png,
    Gif,
    Webp,
    Svg,
}

/*- Functions -*/
/*- Send 404 page -*/
/// Respond with a 404 page, will firstly check
/// if `config.not_found` exists, and grab 404 page path
/// from there, else it will just send 404 as a status code
pub fn not_found(stream: &mut Stream, config: Server) {
    /*- If 404 page is provided -*/
    if let Some(page) = config.not_found {
        stream.respond_file(404u16, page);
    } else {
        stream.respond_status(404u16);
    }
}

/*- Method implementations -*/
impl ResponseType {
    /*- Guesses which response type a file should have -*/
    pub fn guess(path: &Path) -> Self {
        let path: &Path = Path::new(path);

        /*- Check extensions -*/
        match path.extension() {
            Some(ext) => {
                match ext.to_str() {
                    /*- Html -*/
                    Some("html") => ResponseType::Html,
                    Some("htm") => ResponseType::Html,

                    /*- Json -*/
                    Some("json") => ResponseType::Json,
                    Some("yml") => ResponseType::Json,
                    Some("yaml") => ResponseType::Json,

                    /*- Css -*/
                    Some("css") => ResponseType::Css,

                    /*- Js -*/
                    Some("js") => ResponseType::Js,

                    /*- Image -*/
                    Some("png") => ResponseType::Image(ImageType::Png),
                    Some("jpg") => ResponseType::Image(ImageType::Jpeg),
                    Some("jpeg") => ResponseType::Image(ImageType::Jpeg),
                    Some("gif") => ResponseType::Image(ImageType::Gif),
                    Some("webp") => ResponseType::Image(ImageType::Webp),
                    Some("svg") => ResponseType::Image(ImageType::Svg),

                    /*- Text -*/
                    Some(_) => ResponseType::Text,
                    None => ResponseType::Text,
                }
            }
            None => ResponseType::Text,
        }
    }
}
impl Respond {
    /// Construct a request struct
    pub fn new() -> Self {
        Respond {
            response_type: ResponseType::Text,
            content: None,
            additional_headers: None,
        }
    }

    /// Construct a `Respond` struct with text
    ///
    /// ## Examples
    /// ```
    /// use responder::prelude::*;
    ///
    /// Respond::new().text("Hello, world!");
    /// ```
    pub fn text(&mut self, with: &str) -> Self {
        if self.content.is_none() {
            self.response_type = ResponseType::Text;
            self.content = Some(with.to_string());
            self.clone()
        } else {
            panic!("Content buffer already written to");
        }
    }

    /// Construct a `Respond` struct with json
    ///
    /// ## Examples
    /// ```
    /// use responder::prelude::*;
    /// Respond::new().json("{{\"hello\":\"world!\"}}");
    /// ```
    ///
    pub fn json(&mut self, with: &str) -> Self {
        if self.content.is_none() {
            self.response_type = ResponseType::Json;
            self.content = Some(with.to_string());
            self.clone()
        } else {
            panic!("Content buffer already written to");
        }
    }

    /// Construct a `Respond` struct with html
    ///
    /// ## Examples
    /// ```
    /// use responder::prelude::*;
    ///
    /// Respond::new().html("<html><body><h1>Hello!</h1></body></html>");
    /// ```
    ///
    pub fn html(&mut self, with: &str) -> Self {
        if self.content.is_none() {
            self.response_type = ResponseType::Html;
            self.content = Some(with.to_string());
            self.clone()
        } else {
            panic!("Content buffer already written to");
        }
    }

    /// Set additional headers
    pub fn headers(&mut self, headers: Vec<String>) -> Self {
        self.additional_headers = Some(headers);
        self.clone()
    }

    /// Set response type
    pub fn response_type(&mut self, response_type: ResponseType) -> Self {
        self.response_type = response_type;
        self.clone()
    }

    /// Respond with content as a string. Will need response
    /// type as a parameter
    ///
    /// ## Examples
    /// ```
    /// use responder::prelude::*;
    /// use responder::response::ResponseType;
    ///
    /// Respond::new().content("<html><body><h1>Hello!</h1></body></html>", ResponseType::Html);
    /// ```
    ///
    pub fn content(&mut self, with: &str, response_type: ResponseType) -> Self {
        if self.content.is_none() {
            self.response_type = response_type;
            self.content = Some(with.to_string());
            self.clone()
        } else {
            panic!("Content buffer already written to");
        }
    }
}
