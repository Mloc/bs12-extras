/*
  Copyright 2016 Colm Hickey <colmohici@gmail.com>
  Licensed under the Apache License, Version 2.0 (the "License");
  you may not use this file except in compliance with the License.
  You may obtain a copy of the License at
      http://www.apache.org/licenses/LICENSE-2.0
  Unless required by applicable law or agreed to in writing, software
  distributed under the License is distributed on an "AS IS" BASIS,
  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
  See the License for the specific language governing permissions and
  limitations under the License.
*/

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
