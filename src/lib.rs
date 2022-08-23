/*- Global allowings -*/
#![allow(
    unused_imports,
    unused_mut,
    dead_code,
    unused_variables
)]

/*- Module imports -*/
pub mod utils;
pub mod request_info;
pub mod respond;

/*- Imports -*/
pub use request_info::{ RequestInfo, Method };
use terminal_link::Link;
use respond::{
    respond,
    Respond,
    ResponseType,
    not_found,
};
use ansi_term;
use std::{
    net::{
        TcpStream,
        TcpListener
    },
    io::{
        Read, Error,
    },
    thread::spawn,
    path::Path,
    collections::HashMap,
    fs,
};

/*- Constants -*/
const DATA_BUF_INIT:usize = 1024usize;

/*- Structs, enums & unions -*/
#[derive(Clone, Copy)]
pub struct ServerConfig {
    pub addr:       &'static str,
    pub port:       u16,
    pub serve:      Option<&'static str>,
    pub not_found:  Option<&'static str>,
    pub routes:     Route<'static>
}

/*- Send diffrent type of function -*/
#[derive(Clone, Copy)]
pub enum Function {
    /// Function that takes only TcpStream as input,
    S(fn( &mut TcpStream )),
    
    /// Function that takes TcpStream and headers as input,
    SB(fn( &mut TcpStream, &String )),
    
    /// Function that takes TcpStream and headers as input,
    SH(fn( &mut TcpStream, &HashMap<
        &str,
        &str
    > )),

    /// Function that takes TcpStream,
    /// headers and body as params
    SHB(fn(
        &mut TcpStream,
        &HashMap<
            &str,
            &str
        >,
        &String
    )),
}

#[derive(Clone, Copy)]
pub enum Route<'lf> {
    Stack(
        &'lf str,
        &'lf [Route<'lf>]
    ),
    Tail(
        Method,
        &'lf str,
        Function
    )
}

/*- Starting server might fail so return Err(()) if so -*/
pub fn start(__sconfig:ServerConfig) -> Result<(), Error> {
    let bind_to = &format!(
        "{}:{}",
        __sconfig.addr, __sconfig.port
    );

    /*- Start the listener -*/
    let stream = match TcpListener::bind(bind_to) {
        Ok(listener) => listener,

        /*- If failed to open server on port -*/
        Err(e) => return Err(e)
    };

    /*- Log status -*/
    println!("{}", 
        &format!(
            "{} {}",
            ansi_term::Color::RGB(123, 149, 250).paint(
                "Server opened on"
            ),
            ansi_term::Color::RGB(255, 255, 0).underline().paint(
                format!("{}", Link::new(
                    &format!("http://{}", &bind_to),
                    bind_to,
                ))
            )    
        )
    );

    /*- Stream.incoming() is a blocking iterator. Will unblock on requests -*/
    for request in stream.incoming() {

        /*- Spawn a new thread -*/
        spawn(move || {
            /*- Ignore failing requests -*/
            handle_req(match request {
                Ok(req) => req,
                Err(_) => return,
            }, &__sconfig);
        });
    };

    /*- Return, even though it will never happen -*/
    Ok(())
}

/*- Functions -*/
fn handle_req(mut stream:TcpStream, config:&ServerConfig) -> () {
    /*- Data buffer -*/
    let buffer:&mut [u8] = &mut [0u8; DATA_BUF_INIT];

    /*- Read data into buffer -*/
    match stream.read(buffer) {
        Ok(data) => data,
        Err(_) => return
    };

    /*- Parse headers (via utils) -*/
    let request:String = String::from_utf8_lossy(&buffer[..]).to_string();
    let headers:HashMap<&str, &str> = utils::headers::parse_headers(&request);

    /*- Get request info -*/
    let info:RequestInfo = match RequestInfo::parse_req(&request) {
        Ok(e) => e,
        Err(_) => return
    };

    /*- If getting body is neccesary or not -*/
    let mut body:String = String::new();
    if info.method == Method::POST {
        body = request.split("\r\n\r\n").last().unwrap().to_string();
    }

    /*- Get the function or file which is coupled to the request path -*/
    match call_endpoint(&config.routes, info, String::new(), (&headers, &mut stream, &body)) {
        Ok(_) => (),
        Err(_) => {
            /*- If no path was found, we'll check if the
                user want's to serve any static dirs -*/
            if let Some(static_path) = config.serve {
                match serve_static_dir(&static_path, info.path, &stream) {
                    Ok(_) => (),
                    Err(_) => {
                        /*- Now that we didn't find a function, nor
                            a static file, we'll send a 404 page -*/
                        return not_found(&mut stream, *config);
                    }
                };
            }else {
                return not_found(&mut stream, *config);
            };
        },
    };
}

/*- Execute an api function depending on path -*/
fn call_endpoint(
    route:&Route,
    info:RequestInfo,
    mut final_path:String,

    // Function params
    (
        headers,
        stream,
        body
    ):(
        &HashMap<&str, &str>,
        &mut TcpStream,
        &String
    )
) -> Result<(), ()> {

    /*- Check what type of route it is -*/
    match route {
        Route::Stack(pathname, routes) => {

            /*- If a tail was found -*/
            let mut tail_found:bool = false;

            /*- Iterate over all stacks and tails -*/
            'tail_search: for route in routes.iter() {
                let mut possible_final_path = final_path.clone();

                /*- Push the path -*/
                possible_final_path.push_str(pathname);
                possible_final_path.push_str("/");

                /*- Recurse -*/
                match call_endpoint(route, info, possible_final_path.clone(), (headers, stream, body)) {
                    Ok(_) => {
                        tail_found = true;

                        /*- Push the path to the actual final path -*/
                        final_path.push_str(pathname);
                        final_path.push_str("/");
                        break 'tail_search;
                    },
                    Err(_) => continue
                };
            };

            /*- Return -*/
            if tail_found { return Ok(()); }
            else { return Err(()); }
        },
        Route::Tail(method, pathname, function) => {
            /*- If it's the requested path -*/
            if [final_path, pathname.to_string()].concat() == info.path {

                /*- If it's the requested method -*/
                if method == &info.method {
                    /*- Call the associated function -*/
                    Function::call_fn(*function, stream, headers, body);

                    /*- Return success -*/
                    return Ok(());
                }else {
                    return Err(());    
                }
            }else {
                return Err(());
            }
        }
    };
}

/*- Serve static files from a specified dir -*/
fn serve_static_dir(dir:&str, request_path:&str, mut stream:&TcpStream) -> Result<(), ()> {

    /*- Get the requested file path -*/
    let path = &[dir, request_path].concat();
    let file_path:&Path = Path::new(path);

    /*- Check path availability -*/
    match file_path.is_file() {
        true => (),
        false => return Err(())
    };

    /*- Open file -*/
    match fs::File::open(file_path) {
        Ok(mut e) => {
            /*- Get file content -*/
            let mut content:String = String::new();
            match e.read_to_string(&mut content) {
                Ok(_) => (),
                Err(_) => (),
            };

            /*- Respond -*/
            respond(
                &mut stream,
                200u16,
                Some(Respond {
                    response_type: ResponseType::guess(file_path),
                    content
                })
            )
        },
        Err(_) => return Err(())
    }

    Ok(())
}

/*- Method implementation -*/
impl Function {
    pub fn call_fn(self, stream:&mut TcpStream, headers:&HashMap<&str, &str>, body:&String) -> () {
        match self {
            Self::S(e) => e(stream),
            Self::SB(e) => e(stream, body),
            Self::SH(e) => e(stream, headers),
            Self::SHB(e) => e(stream, headers, body),
        }
    }
}
