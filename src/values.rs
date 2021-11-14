use std::collections::HashMap;
#[derive(Debug, PartialEq)]
pub enum Value {
    Str(String),
    Number(i64),
    Float(f64),
    Bool(bool),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Null,
}
