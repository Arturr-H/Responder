/*- Imports -*/
use std::io::Write;

/*- Modules -*/
pub mod headers {

    /*- Imports -*/
    use std::collections::HashMap;

    /*- Parse a data buffer into an hashmap containing headers -*/
    pub fn parse_headers<'lf>(request:&'lf str) -> HashMap<&'lf str, &'lf str> {

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
            match end.insert(k, v) {
                Some(e) => e,
                None => continue
            };
        };

        /*- Return */
        return end;
    }
}