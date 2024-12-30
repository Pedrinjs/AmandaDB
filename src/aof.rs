use std::fs::File;
use std::io::prelude::*;
use std::sync::{Arc, Mutex};

use crate::error::Result;
use crate::handlers::types::Database;
use crate::resp::{reader::RESP, value::Value};

type DB = Arc<Mutex<Database>>;

pub struct AOF {
    file: File,
    insert_queue: Vec<Value>,
}

impl AOF {
    pub fn new(path: impl Into<String>) -> Result<Self> {
        let file = File::options()
            .read(true)
            .write(true)
            .create(true)
            .open(path.into())?;

        Ok(Self { file, insert_queue: Vec::new() })
    }

    pub fn read(&mut self, func: fn(Value, DB), db: DB) -> Result<()> {
        let len = self.file.metadata().unwrap().len();
        if len == 0 {
            return Ok(());
        }

        let mut data = Vec::new();
        self.file.read_to_end(&mut data)?;

        let input = std::str::from_utf8(&data)?;
        let mut reader = RESP::new(&input);
        loop {
            let value = reader.read()?;
            match value {
                Value::Null => break,
                _ => func(value, Arc::clone(&db)),
            };
        }
        Ok(())
    }

    pub fn write(&mut self, value: Value) -> Result<()> {
        self.file.write_all(&value.marshal())?;
        self.file.sync_all()?;
        Ok(())
    }

    pub fn enqueue(&mut self, value: Value) {
        self.insert_queue.push(value);
    }
    pub fn write_queued(&mut self) -> Result<()> {
        for value in self.insert_queue.clone() {
            self.write(value)?;
        }
        self.insert_queue.clear();
        Ok(())
    }
}
