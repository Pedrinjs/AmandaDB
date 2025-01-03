use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::resp::value::Value;

pub type Handler = fn(Vec<Value>, Arc<Mutex<Database>>) -> Value;

pub struct Database {
    set: HashMap<String, String>,
    hset: HashMap<String, HashMap<String, String>>,
    multi: Vec<(Value, Vec<Value>)>,
    transaction_mode: bool,
    execution_mode: bool,
}

impl Database {
    pub fn new() -> Self {
        Self{
            set: HashMap::new(),
            hset: HashMap::new(),
            multi: Vec::new(),
            transaction_mode: false,
            execution_mode: false,
        }
    }

    pub fn is_transaction_mode(&self) -> bool {
        self.transaction_mode
    }
    pub fn set_transaction_mode(&mut self, state: bool) {
        self.transaction_mode = state
    }

    pub fn is_execution_mode(&self) -> bool {
        self.execution_mode
    }
    pub fn set_execution_mode(&mut self, state: bool) {
        self.execution_mode = state
    }

    pub fn set_push(&mut self, key: String, value: String) {
        self.set.insert(key, value);
    }
    pub fn set_get(&self, key: &String) -> Value {
        match self.set.get(key) {
            Some(value) => Value::BulkStr(value.into()),
            None => Value::Null,
        }
    }
    pub fn set_remove(&mut self, key: &String) -> bool {
        match self.set.remove(key) {
            Some(_) => true,
            None => false,
        }
    }
    pub fn set_clear(&mut self) {
        self.set.clear()
    }
    pub fn set_len(&self) -> usize {
        self.set.len()
    }
    pub fn set_incr(&mut self, key: String, num: i64) -> Value {
        let mut value = 0i64;
        let mut err = "";

        self.set.entry(key)
            .and_modify(|val| {
                let v = match val.parse::<i64>() {
                    Ok(n) => n,
                    _ => {
                        err = "ERR: Value is not an integer or out of range";
                        return;
                    },
                };
                value = v + num;
                *val = value.to_string()
            })
            .or_insert_with(|| {
                value += num;
                value.to_string()
            });

        if err.len() != 0 {
            return Value::Error(err);
        }
        Value::Num(value)
    }
    pub fn set_contains(&self, key: &String) -> bool {
        self.set.contains_key(key)
    }

    pub fn hset_push(&mut self, hash: String, key: String, value: String) {
        let map: HashMap<String, String> = HashMap::from([(key, value)]);
        self.hset.insert(hash, map);
    }
    pub fn hset_get(&mut self, hash: &String, key: &String) -> Value {
        let map = match self.hset.get(hash) {
            Some(m) => m.clone(),
            _ => return Value::Null,
        };

        match map.get(key) {
            Some(value) => Value::BulkStr(value.into()),
            None => Value::Null,
        }
    }
    pub fn hset_remove(&mut self, hash: &String, key: &String) -> bool {
        let mut hmap = match self.hset.get(hash) {
            Some(m) => m.clone(),
            None => return false,
        };
        match hmap.remove(key) {
            Some(_) => (),
            None => return false,
        };

        self.hset.insert(hash.into(), hmap);
        true
    }
    pub fn hset_total_len(&self) -> usize {
        let mut len = 0usize;
        for map in self.hset.values() {
            len += map.len();
        }
        len
    }
    pub fn hset_len(&self, hash: &String) -> usize {
        match self.hset.get(hash) {
            Some(m) => m.len(),
            None => 0,
        }
    }
    pub fn hset_clear(&mut self) {
        self.hset.clear()
    }
    pub fn hset_contains(&self, hash: &String, key: &String) -> bool {
        let map = match self.hset.get(hash) {
            Some(m) => m.clone(),
            _ => return false,
        };
        map.contains_key(key)
    }

    pub fn multi_push(&mut self, cmd: Value, args: Vec<Value>) {
        self.multi.push((cmd, args))
    }
    pub fn multi_get(&self) -> Vec<(Value, Vec<Value>)> {
        self.multi.clone()
    }
    pub fn multi_clear(&mut self) {
        self.multi.clear()
    }

    pub fn create_database_copy(&self) -> Self {
        Self {
            set: self.set.clone(),
            hset: self.hset.clone(),
            multi: self.multi.clone(),
            transaction_mode: self.transaction_mode,
            execution_mode: self.execution_mode,
        }
    }
    pub fn database_revert(&mut self, copy: Database) {
        self.set = copy.set;
        self.hset = copy.hset;
        self.multi = copy.multi;
        self.transaction_mode = copy.transaction_mode;
        self.execution_mode = copy.execution_mode;
    }
}
