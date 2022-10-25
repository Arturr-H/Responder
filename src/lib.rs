//! Quickly set up a backend web framework using rust.
//! Very fast and easy to use.

/*- Global allowings -*/
#![allow(
    unused_imports,
    unused_mut,
    dead_code,
    unused_variables
)]

/*- Module imports -*/
pub mod utils;
pub mod response;
pub mod request;
pub mod thread_handler;
pub mod errors;
pub mod stream;

/*- Imports -*/
use crate::response::ResponseType;
use errors::ConfigError;
use request::info::{ RequestInfo, Method };
use terminal_link::Link;
use response::{ Respond, not_found };
use stream::Stream;
use std::{
    net::{
        TcpStream,
        TcpListener
    },
    io::{
        Read, Error,
    },
    path::Path,
    collections::HashMap,
    fs, hash::Hash, num::ParseIntError,
};

/*- Constants -*/
const DATA_BUF_INIT:usize = 1024usize;
const DATA_BUF_POST_INIT:usize = 65536usize;

/*- Structs, enums & unions -*/
/// The Server struct contains changeable fields
/// which configures the server during both startup and
/// whilst it's running.
#[derive(Clone, Copy)]
pub struct Server {
    /// The server address
    addr:       Option<&'static str>,

    /// The server port
    port:       Option<u16>,

    /// The number of threads the current server will use as a maximum
    num_threads:u16,

    /// Serve static files from a directory
    serve:      Option<&'static str>,

    /// Path to a 404 page, if not specified server will return "404 Not Found"
    not_found:  Option<&'static str>,

    /// All http-routes coupled to this server
    routes:     Route,

    /// Check origin & headers before accepting requests
    /// with this function taking headers as input. Will
    /// return a bool indicating wether the request is
    /// valid or not. If it isn't responding will be handled
    /// in the origin control function.
    origin_control:Option<fn(&mut Stream, HashMap<&str, &str>) -> bool>
}

#[derive(Clone, Copy)]
/// A quick way of nesting routes inside of eachother
/// stacks can contain either yet another stack, or a 
/// tail, which will act as an API-endpoint. This enum
/// is used for the server config when initializing the
/// server.
/// 
/// ## Examples
/// ```
/// /*- Initiaize routes -*/
/// let routes = Route::Stack("", &[
///     Route::Stack("nest1", &[
///         Route::Tail(Method::POST, "value", Function::S(|_| {})),
///         Route::Stack("nest2", &[
///             Route::Tail(Method::GET, "value1", Function::S(|_| {})),
///             Route::Tail(Method::GET, "value2", Function::S(|_| {})),
///         ]),
///     ]),
/// ]);
/// ```
pub enum Route {
    Stack(
        &'static str,
        &'static [Route]
    ),
    Tail(
        Method,
        &'static str,
        fn(&mut Stream) -> ()
    )
}

/*- Functions -*/
fn handle_req(stream:TcpStream, config:&Server) {
    /*- Data buffer -*/
    let buffer:&mut Vec<u8> = &mut vec![0u8; DATA_BUF_POST_INIT];
    let mut stream = Stream::from(stream);

    /*- Read data into buffer -*/
    match stream.get_mut_inner_ref().read(buffer) {
        Ok(data) => data,
        Err(_) => return
    };

    /*- Parse headers (via utils) -*/
    let mut request:String = String::from_utf8_lossy(buffer).to_string();
    let headers:HashMap<&str, &str> = utils::headers::parse_headers(&request);

    /*- Check if we should allow origin or not -*/
    match config.origin_control {
        Some(origin_control) => {
            if origin_control(&mut stream, headers.clone()) {
                return
            };
        },
        None => (),
    };

    /*- Get request info -*/
    let mut body:String = String::new();
    let info:RequestInfo = match RequestInfo::parse_req(&request) {
        Ok(e) => e,
        Err(_) => return
    };

    /*- POST requests often contain huge bodies in terms of bytes, (ex when sending images). The
        DATA_BUF_INIT constant is regularly set to a relativly small number like 2048 which images
        won't fit into, therefore we'll update the buffer array to contain more bytes for POST requests -*/
    if info.method == Method::POST {
        body = request.split("\r\n\r\n").last().unwrap().to_string();
        // TODO
    }
    let mut full_path:String = String::new();
    stream.set_body(body);
    stream.set_headers(headers);

    /*- Get the function or file which is coupled to the request path -*/
    match call_endpoint(&config.routes, info, &mut full_path, &mut stream) {
        Ok(_) => (),
        Err(_) => {
            /*- If no path was found, we'll check if the
                user want's to serve any static dirs -*/
            if let Some(static_path) = config.serve {
                match serve_static_dir(static_path, info.path, &mut stream) {
                    Ok(_) => (),
                    Err(_) => {
                        /*- Now that we didn't find a function, nor
                            a static file, we'll send a 404 page -*/
                        not_found(&mut stream, *config);
                    }
                };
            }else {
                not_found(&mut stream, *config);
            };
        },
    };
}

/*- Execute an api function depending on path -*/
fn call_endpoint(
    routes:&Route,
    info:RequestInfo,
    full_path:&mut String,

    /*- Function parameters -*/
    stream: &mut Stream
) -> Result<(), ()> {

    /*- Check what type of route it is -*/
    match routes {
        Route::Stack(pathname, routes) => {

            /*- If a tail was found -*/
            let mut tail_found:bool = false;

            /*- Iterate over all stacks and tails -*/
            'tail_search: for route in routes.iter() {

                /*- Push the path -*/
                let mut possible_full_path = full_path.clone();
                possible_full_path.push_str(pathname);
                possible_full_path.push('/');

                /*- Recurse -*/
                match call_endpoint(route, info, &mut possible_full_path, stream) {
                    Ok(_) => {
                        tail_found = true;

                        /*- Push the path to the actual final path -*/
                        full_path.push_str(pathname);
                        full_path.push('/');
                        break 'tail_search;
                    },
                    Err(_) => continue
                };
            };

            /*- Return -*/
            if tail_found { Ok(()) }
            else { Err(()) }
        },
        Route::Tail(method, pathname, function_ptr) => {

            /*- Store url parameters. An url parameter is a "variable" which
                will be set in the url. Example: localhost:8000/day/:day: -*/
            let mut params:HashMap<String, String> = HashMap::new();

            /*- Push the path to the actual final path -*/
            full_path.push_str(pathname);

            /*- Check for url parameters -*/
            let final_subpaths:Vec<&str> = get_subpaths(full_path);
            let mut final_check_url:String = full_path.clone();

            /*- Iterate and find url params -*/
            for (index, request_path) in get_subpaths(info.path).iter().enumerate() {

                /*- Get the current searched subpath to check wether they are the same -*/
                let subp:&str = match final_subpaths.get(index) {
                    Some(e) => e,
                    None => return Err(())
                };

                match is_url_param(subp) {
                    (true, param_name) => {
                        params.insert(param_name.into(), request_path.to_string());

                        /*- Change full_path -*/
                        final_check_url = final_check_url.replace(subp, request_path);
                        continue;
                    },
                    (false, _) => {
                        if request_path != &subp {
                            return Err(());
                        }else {
                            continue;
                        };
                    },
                }
            }

            /*- If it's the requested path -*/
            if final_check_url == info.path {

                /*- If it's the requested method -*/
                if method == &info.method {

                    /*- Call the associated function -*/
                    stream.set_params(params);
                    function_ptr(stream);

                    /*- Return success -*/
                    Ok(())
                }else {
                    Err(())
                }
            }else {
                Err(())
            }
        },
    }
}

/*- Get subpaths of a full path. Example: get_subpaths("/Path/to/value") -> vec!["Path", "to", "value"] -*/
fn get_subpaths(path:&str) -> Vec<&str> {
    let mut subpaths:Vec<&str> = Vec::new();

    /*- Iterate over all subpaths -*/
    for subpath in path.split('/') {
        if !subpath.is_empty() { subpaths.push(subpath); };
    };

    /*- Return -*/
    subpaths
}

/*- Check if a path is a url parameter -*/
fn is_url_param(path:&str) -> (bool, &str) {
    if path.starts_with(':') && path.ends_with(':') {
        (true, &path[1..path.len()-1])
    }else {
        (false, "")
    }
}

/*- Serve static files from a specified dir -*/
fn serve_static_dir(dir:&str, request_path:&str, stream:&mut Stream) -> Result<(), ()> {

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
        Ok(_) => {
            /*- Get file content -*/
            let mut file_content:String = match fs::read_to_string(file_path) {
                Ok(e) => e,
                Err(_) => {
                    /*- If we can't read the file, we'll send a 404 page -*/
                    return Err(());
                }
            };
            let res:Respond = Respond {
                response_type: ResponseType::guess(file_path),
                content: file_content,
                additional_headers: None
            };

            /*- Respond -*/
            stream.respond(
                200u16,
                Some(res)
            );
        },
        Err(_) => return Err(())
    }

    Ok(())
}

/*- Builder pattern for server config struct -*/
impl<'f> Server {
    pub fn new() -> Server {
        Server {
            addr: None,
            port: None,
            num_threads: 1,
            serve: None,
            not_found: None,
            routes: Route::Stack("", &[]),
            origin_control: None
        }
    }
    pub fn address(&mut self, addr:&'static str) -> &mut Self        { self.addr = Some(addr); self }
    pub fn port(&mut self, port:u16) -> &mut Self                    { self.port = Some(port); self }
    pub fn threads(&mut self, num_threads:u16) -> &mut Self          { self.num_threads = num_threads; self }
    pub fn serve(&mut self, serve:&'static str) -> &mut Self         { self.serve = Some(serve); self }
    pub fn routes(&mut self, routes:Route) -> &mut Self              { self.routes = routes; self }
    pub fn not_found(&mut self, not_found:&'static str) -> &mut Self { self.not_found = Some(not_found); self }
    pub fn origin_control(&mut self, origin_control:fn(&mut Stream, HashMap<&str, &str>) -> bool) -> &mut Self  { self.origin_control = Some(origin_control); self }
    /*- Starting server might fail so return Err(()) if so -*/
    /// Start the server using this function. It takes a 'Server'
    /// struct as input and returns a result, because setting up the
    /// server might fail.
    /// 
    /// ## Example:
    /// ```
    /// Server::new()
    ///     .routes(routes)
    ///     .address("127.0.0.1")
    ///     .port(8080)
    ///     .threads(8)
    ///     .serve("./static")
    ///     .not_found("./static/404.html")
    ///     .start()
    ///     .unwrap();
    /// ```
    pub fn start(self) -> Result<(), ConfigError> {

        /*- Get port and address -*/
        let bind_to = &format!(
            "{}:{}",
            match self.addr {
                Some(e) => e,
                None => return Err(errors::ConfigError::MissingHost)
            },
            match self.port {
                Some(e) => e,
                None => return Err(errors::ConfigError::MissingPort)
            }
        );

        /*- Start the listener -*/
        let stream = match TcpListener::bind(bind_to) {
            Ok(listener) => listener,

            /*- If failed to open server on port -*/
            Err(_) => return Err(ConfigError::HostPortBindingFail)
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

        /*- Initialize thread_handler -*/
        let thread_handler = thread_handler::MainThreadHandler::new(self.num_threads);

        /*- Stream.incoming() is a blocking iterator. Will unblock on requests -*/
        for request in stream.incoming() {
            
            /*- Spawn a new thread -*/
            thread_handler.exec(move || {
                /*- Ignore failing requests -*/
                handle_req(match request {
                    Ok(req) => req,
                    Err(_) => return,
                }, &self);
            });
        };

        /*- Return, even though it will never happen -*/
        Ok(())
    }
}
