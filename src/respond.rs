/*- Imports -*/
use std::{net::TcpStream, io::{Write, Read}, path::Path, fs};
use crate::ServerConfig;

/*- Constants -*/
const STATUS_CODES:&'static [(&'static u16, &'static str); 58] = &[
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
pub struct Respond {
    pub response_type:ResponseType,
    pub content:String
}

/// Decides what the server will respond with
#[derive(Clone, Copy, Debug)]
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
pub fn respond(mut stream:&TcpStream, status:u16, respond:Option<Respond>) -> () {

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
    if let Some(res) = respond {

        /*- Write the status & content to the stream -*/
        match stream.write(
            format!(
                "HTTP/1.1 {}\r\nContent-Length: {}\r\nContent-Type: {}\r\n\r\n{}",
                status, res.content.len(), response_type, res.content
            ).as_bytes()
        ) {
            Ok(_) => (),
            Err(_) => ()
        };
    }else {
        /*- Write the status to the stream -*/
        match stream.write(
            format!(
                "HTTP/1.1 {}\r\n\r\n{} {}",
                status, status, status_msg
            ).as_bytes()
        ) {
            Ok(_) => (),
            Err(_) => ()
        };
    };

    /*- Flush the stream -*/
    match stream.flush() {
        Ok(_) => (),
        Err(_) => ()
    };
}

/*- Send 404 page -*/
pub fn not_found(mut stream:&TcpStream, config:ServerConfig) -> () {
    /*- If 404 page is provided -*/
    if let Some(page) = config.not_found {
        respond(&mut stream, 404u16, file(page));
    }else {
        respond(&mut stream, 404u16, None);
    }
}

/*- Respond with file -*/
pub fn file(path:&str) -> Option<Respond> {

    /*- Grab the path -*/
    let path = Path::new(path);

    /*- Open file -*/
    let content:Option<String> = match fs::File::open(path) {
        Ok(mut e) => {
            let mut content:String = String::new();
            match e.read_to_string(&mut content) {
                Ok(_) => (),
                Err(_) => (),
            };

            Some(content)
        },
        Err(_) => None
    };

    /*- Return -*/
    match content {
        Some(data) => Some(Respond {
            content: data,
            response_type: ResponseType::guess(path)
        }),
        None => None
    }
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
                    Some("html") => return ResponseType::Html,
                    Some("htm")  => return ResponseType::Html,
    
                    /*- Json -*/
                    Some("json") => return ResponseType::Json,
                    Some("yml")  => return ResponseType::Json,
                    Some("yaml") => return ResponseType::Json,

                    /*- Image -*/
                    Some("png")  => return ResponseType::Image(ImageType::Png),
                    Some("jpg")  => return ResponseType::Image(ImageType::Jpeg),
                    Some("jpeg") => return ResponseType::Image(ImageType::Jpeg),
                    Some("gif")  => return ResponseType::Image(ImageType::Gif),
                    Some("webp") => return ResponseType::Image(ImageType::Webp),
                    Some("svg")  => return ResponseType::Image(ImageType::Svg),
     
                    /*- Text -*/
                    Some(_)   => return ResponseType::Text,
                    None      => return ResponseType::Text,
                };
            },
            None => return ResponseType::Text,
        };
    }
}

