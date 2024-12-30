use super::constants::*;

#[derive(Clone)]
pub enum Value {
    Str(&'static str),
    Error(&'static str),
    Num(i64),
    Bulk(String),
    Array(Vec<Value>),
    Null,
}

impl Value {
    pub fn marshal(self) -> Vec<u8> {
        match self {
            Value::Str(_) => self.marshal_string(),
            Value::Error(_) => self.marshal_error(),
            Value::Num(_) => self.marshal_number(),
            Value::Bulk(_) => self.marshal_bulk(),
            Value::Array(_) => self.marshal_array(),
            Value::Null => self.marshal_null(),
        }
    }

    fn marshal_string(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        if let Value::Str(s) = self {
            bytes.push(STRING);
            bytes.extend(s.as_bytes());
            bytes.extend(['\r' as u8, '\n' as u8]);
        }
        bytes
    }
    
    fn marshal_error(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        if let Value::Error(e) = self {
            bytes.push(ERROR);
            bytes.extend(e.as_bytes());
            bytes.extend(['\r' as u8, '\n' as u8]);
        }
        bytes
    }

    fn marshal_number(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        if let Value::Num(n) = self {
            bytes.push(NUMBER);
            bytes.extend(n.to_string().as_bytes());
            bytes.extend(['\r' as u8, '\n' as u8]);
        }
        bytes
    }

    fn marshal_bulk(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        if let Value::Bulk(bulk) = self {
            bytes.push(BULK);
            let len = std::char::from_digit(bulk.chars().count() as u32, 10).unwrap() as u8;
            bytes.extend([len, '\r' as u8, '\n' as u8]);
            bytes.extend(bulk.as_bytes());
            bytes.extend(['\r' as u8, '\n' as u8]);
        }
        bytes
    }

    fn marshal_array(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        if let Value::Array(arr) = self {
            bytes.push(ARRAY);
            let len = std::char::from_digit(arr.len() as u32, 10).unwrap() as u8;
            bytes.extend([len, '\r' as u8, '\n' as u8]);
            arr.into_iter()
                .for_each(|value| bytes.extend(value.marshal()));
        }
        bytes
    }

    fn marshal_null(self) -> Vec<u8> {
        "$-1\r\n".as_bytes().to_owned()
    }
}
