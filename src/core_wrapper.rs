extern crate serde_json;
extern crate madder_core;
use madder_core::*;

pub fn as_properties(value: &serde_json::Value) -> Properties {
    value.as_array().unwrap().iter().map(|v| {
        let v_arr = v.as_array().unwrap();
        (v_arr[0], v_arr[1])
    }).collect()
}

