extern crate serde_json;
use serde_json::Value;

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
                let path = hmap["path"].as_str().map(&parse_path);
                let from = hmap["from"].as_str().map(&parse_path);
                let value = hmap["value"].clone();

                match op.unwrap() {
                    "add" => Ok(Add(path.unwrap(), value)),
                    "remove" => Ok(Remove(path.unwrap())),
                    "copy" => Ok(Copy(path.unwrap(), from.unwrap())),
                    "move" => Ok(Move(path.unwrap(), from.unwrap())),
                    "test" => Ok(Test(path.unwrap(), value)),
                    "replace" => Ok(Replace(path.unwrap(), value)),
                    _ => unimplemented!(),
                }
            },
            _ => unimplemented!(),
        }
    }
}

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

