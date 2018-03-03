extern crate serde_json;
use serde_json::Value;

pub enum IndexRange {
    Index(usize),
    ReverseIndex(usize),
    All,
}

impl IndexRange {
    pub fn from_str(raw: &str) -> Option<IndexRange> {
        if raw == "-" {
            Some(IndexRange::All)
        } else {
            raw.parse::<i32>().ok().map(|n| if n >= 0 { IndexRange::Index(n as usize) } else { IndexRange::ReverseIndex(n.abs() as usize) })
        }
    }
}

pub enum PointerElement {
    IndexElement(IndexRange),
    KeyElement(String),
}

type Pointer = Vec<String>;

pub enum Operation {
    Add(Pointer, Value),
    Remove(Pointer),
    Copy(Pointer, Pointer),
    Move(Pointer, Pointer),
    Test(Pointer, Value),
    Replace(Pointer, Value),
}

impl Operation {
    pub fn new_from_json(json: Value) -> Result<Operation, PatchError> {
        use self::Operation::*;

        match json {
            Value::Object(hmap) => {
                let parse_path = |path: &str| {
                    path.split('/').skip(1).map(|x| x.replace("~1", "/").replace("~0", "~")).collect::<Vec<_>>()
                };

                let op = hmap["op"].as_str();
                let path = hmap.get("path").and_then(|ref v| v.as_str()).map(&parse_path);
                let from = hmap.get("from").and_then(|ref v| v.as_str()).map(&parse_path);
                let value = hmap.get("value").cloned();

                match op.unwrap() {
                    "add" => Ok(Add(path.unwrap(), value.unwrap())),
                    "remove" => Ok(Remove(path.unwrap())),
                    "copy" => Ok(Copy(path.unwrap(), from.unwrap())),
                    "move" => Ok(Move(path.unwrap(), from.unwrap())),
                    "test" => Ok(Test(path.unwrap(), value.unwrap())),
                    "replace" => Ok(Replace(path.unwrap(), value.unwrap())),
                    _ => unimplemented!(),
                }
            },
            _ => unimplemented!(),
        }
    }

    pub fn get_path(&self) -> Pointer {
        use self::Operation::*;

        match self {
            &Add(ref path,_) => path.clone(),
            &Remove(ref path) => path.clone(),
            &Copy(ref path,_) => path.clone(),
            &Move(ref path,_) => path.clone(),
            &Test(ref path,_) => path.clone(),
            &Replace(ref path,_) => path.clone(),
        }
    }
}

#[derive(Debug)]
pub enum PatchError {
}

pub trait Patch {
    fn patch_once(&mut self, op: Operation) -> Result<(), PatchError>;

    fn patch(&mut self, ops: Vec<Operation>) -> Result<(), PatchError> {
        for op in ops {
            if let Err(err) = self.patch_once(op) {
                return Err(err);
            }
        }

        Ok(())
    }
}
