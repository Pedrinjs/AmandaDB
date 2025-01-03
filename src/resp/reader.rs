use std::io::{BufReader, Bytes, Read};

use super::{constants::{ARRAY, BULKSTR, CR}, value::Value};
use crate::error::Result;

pub struct RESP<'a> {
    reader: Bytes<BufReader<&'a [u8]>>,
}

impl<'a> RESP<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            reader: BufReader::new(input.as_bytes()).bytes(),
        }
    }

    fn read_line(&mut self) -> Vec<u8> {
        let mut line: Vec<u8> = Vec::new();
        while let Some(Ok(b)) = self.reader.next() {
            line.push(b);
            
            if line.len() >= 2 && line[line.len()-2] == CR {
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
            BULKSTR => self.read_bulk()?,
            _ => {
                println!("Unknwon type: {:?}", _type as char);
                Value::Null
            }
        })
    }

    fn read_array(&mut self) -> Result<Value> {
        let len = self.read_integer()?;
        let mut value: Vec<Value> = Vec::new();

        for _ in 0..len {
            let temp = self.read()?;
            value.push(temp);
        }

        Ok(Value::Array(value))
    }

    fn read_bulk(&mut self) -> Result<Value> {
        let _len = self.read_integer()?;
        let value = String::from_utf8_lossy(&self.read_line()).to_string();
        Ok(Value::BulkStr(value))
    }
}
