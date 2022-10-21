
/*- Modules -*/
pub mod headers {

    /*- Imports -*/
    use std::collections::HashMap;

    /*- Parse a data buffer into an hashmap containing headers -*/
    pub fn parse_headers(request:&str) -> HashMap<&str, &str> {

        /*- Create the hashmap -*/
        let mut end:HashMap<&str, &str> = HashMap::new();

        /*- Iterate over lines -*/
        for line in request.split("\r\n") {
            let (k, v) = match line.split_once(':') {
                Some(e) => e,
                None => {
                    continue;
                }
            };

            /*- Add k and v to hashmap -*/
            match end.insert(k, v.trim_start()) {
                Some(e) => e,
                None => continue
            };
        };

        /*- Return */
        end
    }
}

/*- Controlling request flow -*/
pub mod control_flow {
    use std::net::TcpStream;
    use crate::response::{ respond, Respond };

    pub fn redirect(stream:&mut TcpStream, url:&str) -> () {
        println!("{url}");
        respond(
            stream,
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


