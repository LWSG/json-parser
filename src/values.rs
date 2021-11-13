use std::collections::HashMap;
#[derive(Debug, PartialEq)]
pub enum Value {
    Str(String),
    Number(i32),
    Bool(bool),
    Array(Vec<Value>),
    Object(HashMap<String, Value>),
    Null,
}
