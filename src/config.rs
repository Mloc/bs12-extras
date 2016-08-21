use std::collections::HashMap;
use std::path::Path;
use std::fs::File;
use std::io::Read;

use rustc_serialize::json;

pub fn load_config<P: AsRef<Path>>(path: P) -> Config {
    let mut f = File::open(path).unwrap();
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();
    json::decode(&buf).unwrap()
}

#[derive(RustcDecodable, Debug)]
pub struct Config {
    pub zmq_endpoint: String,

    pub blacklists: Vec<BlacklistEntry>,
}

#[derive(RustcDecodable, Debug, Clone)]
pub struct BlacklistEntry {
    pub domain: String,
    pub message: String,
    pub reasons: HashMap<String, String>,
}
