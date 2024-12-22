use std::char::from_digit;
use std::io::{BufReader, Bytes, prelude::*};

use crate::error::Result;

const STRING: u8 = '+' as u8;
const ERROR: u8 = '-' as u8;
const INTEGER: u8 = ':' as u8;
const BULK: u8 = '$' as u8;
const ARRAY: u8 = '*' as u8;

#[derive(Clone, Debug)]
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

        bytes.push(STRING);
        if let Value::Str(s) = self {
            let temp = s.as_bytes();
            bytes.append(&mut temp.to_vec());
        }
        bytes.extend(['\r' as u8, '\n' as u8]);
        bytes
    }

    fn marshal_error(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.push(ERROR);
        if let Value::Error(e) = self {
            let temp = e.as_bytes();
            bytes.append(&mut temp.to_vec());
        }
        bytes.extend(['\r' as u8, '\n' as u8]);
        bytes
    }

    fn marshal_number(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();
        bytes.push(INTEGER);
        if let Value::Num(n) = self {
            bytes.append(&mut n.to_string().into_bytes());
        }
        bytes.push('\r' as u8);
        bytes.push('\n' as u8);
        bytes
    }

    fn marshal_bulk(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.push(BULK);
        if let Value::Bulk(bulk) = self {
            let len = from_digit(bulk.chars().count() as u32, 10).unwrap() as u8;
            bytes.push(len);
            bytes.extend(['\r' as u8,'\n' as u8]);
            bytes.append(&mut bulk.into_bytes());
        }
        bytes.extend(['\r' as u8, '\n' as u8]);
        bytes
    }

    fn marshal_array(self) -> Vec<u8> {
        let mut bytes: Vec<u8> = Vec::new();

        bytes.push(ARRAY);
        if let Value::Array(arr) = self {
            let len = from_digit(arr.len() as u32, 10).unwrap() as u8;
            bytes.push(len);
            bytes.extend(['\r' as u8, '\n' as u8]);

            for value in arr {
                bytes.append(&mut value.marshal());
            }
        }
        bytes
    }

    fn marshal_null(self) -> Vec<u8> {
        String::from("$-1\r\n").into_bytes()
    }
}

pub struct Resp<'a> {
    reader: Bytes<BufReader<&'a [u8]>>,
}

impl<'a> Resp<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            reader: BufReader::new(input.as_bytes()).bytes(),
        }
    }

    fn read_line(&mut self) -> Vec<u8> {
        let mut line: Vec<u8> = Vec::new();
        while let Some(Ok(b)) = self.reader.next() {
            line.push(b);
            
            if line.len() >= 2 && line[line.len()-2] == ('\r' as u8) {
                break;
            }
        }

        line[0..line.len()-2].to_vec()
    }

    fn read_integer(&mut self) -> Result<i64> {
        let line = self.read_line();
        let string = String::from_utf8_lossy(&line);
        
        let int = string.parse::<i64>()?;
        Ok(int)
    }

    pub fn read(&mut self) -> Result<Value> {
        let _type = match self.reader.next() {
            Some(Ok(t)) => t,
            _ => { return Ok(Value::Null); },
        };

        Ok(match _type {
            ARRAY => self.read_array()?,
            BULK => self.read_bulk()?,
            _ => {
                println!("Unknwon type: {:?}", _type as char);
                Value::Null
            }
        })
    }

    fn read_array(&mut self) -> Result<Value> {
        let len = self.read_integer()?;
        let mut value: Vec<Value> = Vec::new();

        for _ in 1..=len {
            let temp = self.read()?;
            value.push(temp);
        }

        Ok(Value::Array(value))
    }

    fn read_bulk(&mut self) -> Result<Value> {
        let _len = self.read_integer()?;
        let value = String::from_utf8_lossy(&self.read_line()).to_string();
        Ok(Value::Bulk(value))
    }
}

pub struct Writer {
    writer: Box<dyn Write>,
}

impl Writer {
    pub fn new(writer: Box<dyn Write>) -> Self {
        Writer { writer }
    }

    pub fn write(&mut self, value: Value) -> Result<()> {
        let bytes = value.marshal();
        self.writer.write_all(&bytes)?;
        self.writer.flush()?;
        Ok(())
    }
}
