/*- Imports -*/
use std::{ io::{ Write, Read }, path::Path, fs };
use crate::{ Server, stream::Stream };

/*- Constants -*/
pub const STATUS_CODES:&[(&u16, &str); 58] = &[
    (&400, "Bad Request"),                      (&500, "Internal Server Error"),
    (&401, "Unauthorized"),                     (&501, "Not Implemented"),
    (&402, "Payment Required"),                 (&502, "Bad Gateway"),
    (&403, "Forbidden"),                        (&503, "Service Unavailable"),          /*=-----------=*/
    (&404, "Not Found"),                        (&504, "Gateway Timeout"),              //             \\
    (&405, "Method Not Allowed"),               (&505, "HTTP Version Not Supported"),   //     500     \\
    (&406, "Not Acceptable"),                   (&506, "Variant Also Negotiates"),      //             \\
    (&407, "Proxy Authentication Required"),    (&507, "Insufficient Storage"),         /*=-----------=*/
    (&408, "Request Timeout"),                  (&508, "Loop Detected"),
    (&409, "Conflict"),                         (&510, "Not Extended"),
    (&410, "Gone"),                             (&511, "Network Authentication Required"),
    (&411, "Length Required"),                              (&200, "OK"),
    (&412, "Precondition Failed"),                          (&201, "Created"),
    (&413, "Payload Too Large"),           /* 200 OK -> */  (&202, "Accepted"),
    (&414, "URI Too Long"),                /* 200 OK -> */  (&204, "No Content"),
    (&415, "Unsupported Media Type"),      /* 200 OK -> */  (&205, "Reset Content"),
    (&416, "Range Not Satisfiable"),       /* 200 OK -> */  (&206, "Partial Content"),
    (&417, "Expectation Failed"),          /* 200 OK -> */  (&207, "Multi-status"),
    (&418, "I'm a teapot"),                                 (&208, "Already reported"), 
    (&421, "Misdirected Request"),                          (&226, "IM Used"),
    (&422, "Unprocessable Entity"),             (&300, "Multiple Choices"),
    (&423, "Locked"),                           (&301, "Moved Permanently"),
    (&424, "Failed Dependency"),                (&302, "Found"),                    /*=-----------=*/
    (&425, "Too Early"),                        (&303, "See Other"),                //             \\
    (&426, "Upgrade Required"),                 (&304, "Not Modified"),             //     300     \\
    (&428, "Precondition Required"),            (&305, "Use Proxy"),                //             \\
    (&429, "Too Many Requests"),                (&306, "Switch Proxy"),             /*=-----------=*/
    (&431, "Request Header Fields Too Large"),  (&307, "Temporary Redirect"),
    (&451, "Unavailable For Legal Reasons"),    (&308, "Permanent Redirect"),
];

/*- Structs, enums & unions -*/
#[derive(Clone, Debug)]
/// The respond function takes an optional Respond struct
/// as input, which will contain a content type and content
pub struct Respond {
    pub response_type:ResponseType,
    pub content:      String,
    pub additional_headers:Option<Vec<String>>
}

#[derive(Clone, Copy, Debug)]
/// Decides what the server will respond with
pub enum ResponseType {
    Text,
    Json,
    Html,
    Image(ImageType),
}

/// Server can also respond with images
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
/// Quickly repond with a 404 page, will firstly check
/// if config.not_found exists, and grab 404 page path
/// from there, else it will just send 404 as a status code
/// 
/// ## Example
/// ```
/// not_found(&mut stream, config);
/// ```
pub fn not_found(stream:&mut Stream, config:Server) {
    /*- If 404 page is provided -*/
    if let Some(page) = config.not_found {
        stream.respond(404u16, with_file(page));
    }else {
        stream.respond(404u16, None);
    }
}

/*- Respond with file -*/
/// Will return a Respond struct containing information
/// to send a file, like an image, text, or json file
/// 
/// ## Example
/// ```
/// respond(&mut stream, 200u16, with_file("/path/to/file.png"))
/// ```
pub fn with_file(path:&str) -> Option<Respond> {

    /*- Grab the path -*/
    let path = Path::new(path);

    /*- Open file -*/
    let content:Option<String> = match fs::File::open(path) {
        Ok(mut e) => {
            let mut content:String = String::new();
            if e.read_to_string(&mut content).is_ok() { }

            Some(content)
        },
        Err(_) => None
    };

    /*- Return -*/
    content.map(|data| Respond {
        content: data,
        response_type: ResponseType::guess(path),
        additional_headers: None
    })
}

/*- Method implementations -*/
impl ResponseType {
    /*- Guesses which response type a file should have -*/
    pub fn guess(path:&Path) -> Self {
        let path:&Path = Path::new(path);

        /*- Check extensions -*/
        match path.extension() {
            Some(ext) => {
                match ext.to_str() {
                    /*- Html -*/
                    Some("html") => ResponseType::Html,
                    Some("htm")  => ResponseType::Html,
    
                    /*- Json -*/
                    Some("json") => ResponseType::Json,
                    Some("yml")  => ResponseType::Json,
                    Some("yaml") => ResponseType::Json,

                    /*- Image -*/
                    Some("png")  => ResponseType::Image(ImageType::Png),
                    Some("jpg")  => ResponseType::Image(ImageType::Jpeg),
                    Some("jpeg") => ResponseType::Image(ImageType::Jpeg),
                    Some("gif")  => ResponseType::Image(ImageType::Gif),
                    Some("webp") => ResponseType::Image(ImageType::Webp),
                    Some("svg")  => ResponseType::Image(ImageType::Svg),
     
                    /*- Text -*/
                    Some(_)   => ResponseType::Text,
                    None      => ResponseType::Text,
                }
            },
            None => ResponseType::Text,
        }
    }
}
impl Respond {
    /// Quickly respond with text
    /// 
    /// ## Examples
    /// respond(&mut stream, 200u16, Respond::text("Hello world!"))
    pub fn text(with:&str) -> Option<Respond> {
        Some(
            Respond { response_type: ResponseType::Text, content: with.to_string(), additional_headers: None }
        )
    }

    /// Respond without any content whatsoever
    pub fn empty() -> Option<()> {
        None
    }
}
