extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;
extern crate serde_yaml;
extern crate madder_core;
use madder_core::*;
use std::fs;
use std::io::BufReader;
use std::io::prelude::*;

pub struct App {
    madder: Madder,
}

impl App {
    pub fn new() -> App {
        App {
            madder: Madder::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct Request {
    method: Method,
    path: String,
    entity: serde_json::Value,
}

// for yaml
#[derive(Serialize, Deserialize)]
pub struct AppYaml {
    #[serde(flatten)]
    project: serde_yaml::Value,

    #[serde(default = "Vec::new")]
    operations: Vec<Request>,
}

fn main() {
    let args: Vec<String> = ::std::env::args().collect();

    if args.len() >= 2 {
        let file = fs::File::open(&args[1]).unwrap();
        let mut buf_reader = BufReader::new(file);
        let mut contents = String::new();
        buf_reader.read_to_string(&mut contents).unwrap();

        let yaml = serde_yaml::from_str::<AppYaml>(&contents).unwrap();

        let mut app = App::new();
        app.madder.from_yaml(yaml.project).unwrap();

        for op in yaml.operations {
            app.madder.request(op.method, &op.path, op.entity).unwrap();
        }

        println!("{}", serde_yaml::to_string(&app.madder.to_yaml().unwrap()).unwrap());
    }
}


