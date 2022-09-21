
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