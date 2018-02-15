extern crate gstreamer as gst;
extern crate serde;
extern crate serde_json;

use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;

#[derive(Serialize, Deserialize)]
pub struct EditorStructure {
    pub size: (i32, i32),
    pub components: Box<[ComponentStructure]>,
}

impl EditorStructure {
    pub fn new_from_file(file: &str) -> EditorStructure {
        let file = File::open(file).unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents).unwrap();

        serde_json::from_str(&contents).unwrap()
    }
}

#[derive(Serialize, Deserialize)]
pub struct ComponentStructure {
    pub component_type: ComponentType,
    pub start_time: u64,
    pub end_time: u64,
    pub entity: String,
    pub coordinate: (i32, i32),
}

#[derive(Serialize, Deserialize)]
pub enum ComponentType {
    Video,
    Image,
    Text,
    Sound,
}

