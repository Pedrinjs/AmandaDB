use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::resp::Value;

pub type Handler = fn(Vec<Value>, db: Arc<Mutex<Database>>) -> Value;

pub struct Database {
    pub set: HashMap<String, String>,
    pub hset: HashMap<String, HashMap<String, String>>,
    pub multi: Vec<(Vec<Value>, Handler)>,
    transaction_mode: bool,
}

impl Database {
    pub fn new() -> Self {
        Self{
            set: HashMap::new(),
            hset: HashMap::new(),
            multi: Vec::new(),
            transaction_mode: false,
        }
    }

    pub fn is_transaction_mode(&self) -> bool {
        self.transaction_mode
    }

    pub fn set_transaction_mode(&mut self, state: bool) {
        self.transaction_mode = state
    }
}
