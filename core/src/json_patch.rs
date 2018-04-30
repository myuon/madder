use std::ops::Add;

extern crate serde_json;
use serde_json::Value;

#[derive(Clone)]
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

pub trait AsIndexRange {
    type Output;
    fn as_index(&self, index: IndexRange) -> &Self::Output;
    fn as_index_mut(&mut self, index: IndexRange) -> &mut Self::Output;
}

impl<T> AsIndexRange for Vec<T> {
    type Output = T;

    fn as_index(&self, index: IndexRange) -> &Self::Output {
        use IndexRange::*;

        match index {
            Index(i) => &self[i],
            ReverseIndex(i) => {
                let n = self.len();
                &self[n-i]
            },
            _ => unimplemented!(),
        }
    }

    fn as_index_mut(&mut self, index: IndexRange) -> &mut Self::Output {
        use IndexRange::*;

        match index {
            Index(i) => &mut self[i],
            ReverseIndex(i) => {
                let n = self.len();
                &mut self[n-i]
            },
            _ => unimplemented!(),
        }
    }
}

pub enum PointerElement {
    IndexElement(IndexRange),
    KeyElement(String),
}

#[derive(Debug, Clone)]
pub struct Pointer(pub Vec<String>);

impl Pointer {
    pub fn from_str(path: &str) -> Pointer {
        Pointer(path.split('/').skip(1).map(|x| x.replace("~1", "/").replace("~0", "~")).collect::<Vec<_>>())
    }
}

impl Add for Pointer {
    type Output = Pointer;

    fn add(self, rhs: Self) -> Self::Output {
        let mut vec = self.0;
        let mut vec_r = rhs.0;
        vec.append(&mut vec_r);
        Pointer(vec)
    }
}

// JSON Patch API
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
                let op = hmap["op"].as_str();
                let path = hmap.get("path").and_then(|ref v| v.as_str()).map(&Pointer::from_str);
                let from = hmap.get("from").and_then(|ref v| v.as_str()).map(&Pointer::from_str);
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
    type ContentType : Clone;

    fn patch_once(&mut self, op: Operation, content_type: Self::ContentType) -> Result<(), PatchError>;

    fn patch(&mut self, ops: Vec<Operation>, content_type: Self::ContentType) -> Result<(), PatchError> {
        for op in ops {
            if let Err(err) = self.patch_once(op, content_type.clone()) {
                return Err(err);
            }
        }

        Ok(())
    }

    fn get_by_pointer(&self, path: Pointer, content_type: Self::ContentType) -> Value;
}

