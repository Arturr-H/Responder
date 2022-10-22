/*- Structs, enums & unions -*/
#[derive(Debug)]
pub enum ConfigError {
    MissingPort,
    MissingHost,
    HostPortBindingFail
}
