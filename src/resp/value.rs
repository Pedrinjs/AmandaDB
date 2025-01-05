use std::collections::{HashMap, HashSet};

use super::constants::*;

fn usize_to_u8(number: usize) -> u8 {
    match char::from_digit(number as u32, 10) {
        Some(c) => c as u8,
        None => 0,
    }
}

#[derive(Clone)]
pub enum Value {
    Str(&'static str),
    Error(&'static str),
    Num(i64),
    BulkStr(String),
    Array(Vec<Value>),
    Bool(bool),
    Double(f64),
    // BigNum(i128), // no big numbers for you
    BulkError(String),
    // Verbatim(String), // r"no verbatim"
    Map(HashMap<Value, Value>),
    Attr(HashMap<Value, Value>),
    Set(HashSet<Value>),
    Push(Vec<Value>),
    Null,
}

impl Value {
    pub fn is_error(&self) -> Option<&str> {
        if let Value::Error(e) = self {
            return Some(e);
        }
        None
    }

    pub fn marshal(self) -> Vec<u8> {
        match self {
            Value::Str(_) => self.marshal_string(),
            Value::Error(_) => self.marshal_error(),
            Value::Num(_) => self.marshal_number(),
            Value::BulkStr(_) => self.marshal_bulkstr(),
            Value::Array(_) => self.marshal_array(),
            Value::Bool(_) => self.marshal_bool(),
            Value::Double(_) => self.marshal_double(),
            // Value::BigNum(_) => self.marshal_bignum(),
            Value::BulkError(_) => self.marshal_bulkerr(),
            // Value::Verbatim(_) => self.marshal_verbatim(),
            Value::Map(_) => self.marshal_map(),
            Value::Attr(_) => self.marshal_attr(),
            Value::Set(_) => self.marshal_set(),
            Value::Push(_) => self.marshal_push(),
            Value::Null => self.marshal_null(),
        }
    }

    fn marshal_string(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        if let Value::Str(s) = self {
            bytes.push(STRING);
            bytes.extend(s.as_bytes());
            bytes.extend([CR, LF]);
        }
        bytes
    }
    
    fn marshal_error(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        if let Value::Error(e) = self {
            bytes.push(ERROR);
            bytes.extend(e.as_bytes());
            bytes.extend([CR, LF]);
        }
        bytes
    }

    fn marshal_number(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        if let Value::Num(n) = self {
            bytes.push(NUMBER);
            bytes.extend(n.to_string().as_bytes());
            bytes.extend([CR, LF]);
        }
        bytes
    }

    fn marshal_bulkstr(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        if let Value::BulkStr(blk) = self {
            bytes.push(BULKSTR);
            let len = usize_to_u8(blk.chars().count());
            bytes.extend([len, CR, LF]);
            bytes.extend(blk.as_bytes());
            bytes.extend([CR, LF]);
        }
        bytes
    }

    fn marshal_array(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        if let Value::Array(arr) = self {
            bytes.push(ARRAY);
            let len = usize_to_u8(arr.len());
            bytes.extend([len, CR, LF]);
            arr.into_iter().for_each(|value| bytes.extend(value.marshal()));
        }
        bytes
    }

    fn marshal_bool(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        if let Value::Bool(bol) = self {
            bytes.push(BOOLEAN);
            match bol {
                true => bytes.push('t' as u8),
                false => bytes.push('f' as u8),
            }
            bytes.extend([CR, LF]);
        }
        bytes
    }

    fn marshal_double(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        if let Value::Double(double) = self {
            bytes.push(DOUBLE);
            bytes.extend(double.to_string().as_bytes());
            bytes.extend([CR, LF]);
        }
        bytes
    }

    /*fn marshal_bignum(self) -> Vec<u8> {
        Vec::new()
    }*/

    fn marshal_bulkerr(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        if let Value::BulkError(bulk) = self {
            bytes.push(BULKERR);
            let len = usize_to_u8(bulk.chars().count());
            bytes.extend([len, CR, LF]);
            bytes.extend(bulk.as_bytes());
            bytes.extend([CR, LF]);
        }
        bytes
    }

    /*fn marshal_verbatim(self) -> Vec<u8> {
        Vec::new()
    }*/

    fn marshal_map(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        if let Value::Map(map) = self {
            bytes.push(MAP);
            let len = usize_to_u8(map.len());
            bytes.extend([len, CR, LF]);
            map.into_iter().for_each(|(key, value)| {
                bytes.extend(key.marshal());
                bytes.extend(value.marshal());
            });
        }
        bytes
    }

    fn marshal_attr(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        if let Value::Attr(attr) = self {
            bytes.push(ATTRIBUTE);
            let len = usize_to_u8(attr.len());
            bytes.extend([len, CR, LF]);
            attr.into_iter().for_each(|(key, value)| {
                bytes.extend(key.marshal());
                bytes.extend(value.marshal());
            });
        }
        bytes
    }

    fn marshal_set(self) -> Vec<u8>{
        let mut bytes: Vec<u8> = Vec::new();
        if let Value::Set(set) = self {
            bytes.push(SET);
            let len = usize_to_u8(set.len());
            bytes.extend([len, CR, LF]);
            set.into_iter().for_each(|value| bytes.extend(value.marshal()));
        }
        bytes
    }

    fn marshal_push(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        if let Value::Push(push) = self {
            bytes.push(PUSH);
            let len = usize_to_u8(push.len());
            bytes.extend([len, CR, LF]);
            push.into_iter().for_each(|value| bytes.extend(value.marshal()));
        }
        bytes
    }

    fn marshal_null(self) -> Vec<u8> {
        "$_\r\n".as_bytes().to_owned()
    }
}
