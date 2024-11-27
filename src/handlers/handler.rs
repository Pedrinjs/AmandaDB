use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::types::{Database, Handler};
use crate::aof::AOF;
use crate::resp::Value;

type Aof = Arc<Mutex<AOF>>;
type DB = Arc<Mutex<Database>>;

pub struct Handlers<'a> {
    handlers: HashMap<&'a str, Handler>,
}

impl<'a> Handlers<'a> {
    pub fn new() -> Self {
        Handlers{
            handlers: HashMap::new(),
        }
    }

    pub fn match_handler(&mut self, input: Value, aof: Aof, db: DB) -> Value {
        let Value::Array(arr) = input.clone() else {
            return Value::Error("ERR: Only arrays should be used");
        };
        if arr.len() == 0 {
            return Value::Error("ERR: An empty array was provided");
        }

        let Value::Bulk(command) = &arr[0] else {
            return Value::Error("ERR: The command must be a bulk string");
        };

        let temp = command.to_uppercase();
        let cmd = temp.as_str();

        if let Some(handler) = self.get(cmd) {
            if cmd == "EXEC" || cmd == "DISCARD" {
                db.lock().unwrap().set_transaction_mode(false);
            }

            let args = &arr[1..];
            if db.lock().unwrap().is_transaction_mode() {
                db.lock().unwrap().multi.push((args.to_vec(), *handler));
                return Value::Str("QUEUED");
            }

            let command_list = vec!["SET", "HSET", "DEL", "HDEL", "INCR", "INCRBY", "DECR", "DECRBY"];
            if command_list.contains(&cmd) {
                match aof.lock().unwrap().write(input) {
                    Err(_) => return Value::Error("ERR: Failed to append to AOF"),
                    _ => (),
                };
            }
            return handler(args.to_vec(), db);
        } else {
            return Value::Error("ERR: Command does not exist");
        }
    }

    fn insert(&mut self, key: &'a str, handler: Handler) {
        self.handlers.insert(key, handler);
    }

    pub fn get(&self, key: &'a str) -> Option<&Handler> {
        self.handlers.get(key)
    }

    pub fn init(&mut self) {
        self.insert("PING", ping);
        self.insert("ECHO", echo);
        self.insert("DBSIZE", dbsize);
        self.insert("FLUSHDB", flushdb);
        self.insert("EXISTS", exists);
        self.insert("HEXISTS", hexists);
        self.insert("SET", set);
        self.insert("HSET", hset);
        self.insert("GET", get);
        self.insert("HGET", hget);
        self.insert("DEL", del);
        self.insert("HDEL", hdel);
        self.insert("INCR", incr);
        self.insert("INCRBY", incr_by);
        self.insert("DECR", decr);
        self.insert("DECRBY", decr_by);
        self.insert("MULTI", multi);
        self.insert("EXEC", exec);
        self.insert("DISCARD", discard);
    }
}

fn ping(args: Vec<Value>, _db: DB) -> Value {
    if args.len() > 1 {
        return Value::Error("ERR: Wrong number of arguments for command");
    }
    if args.len() == 0 {
        return Value::Str("PONG");
    }

    let Value::Bulk(name) = &args[0] else {
        return Value::Str("PONG");
    };
    return Value::Bulk(name.into());
}

fn echo(args: Vec<Value>, _db: DB) -> Value {
    if args.len() != 1 {
        return Value::Error("ERR: Wrong number of arguments provided");
    }

    if let Value::Bulk(name) = &args[0] {
        return Value::Bulk(name.into());
    }
    return Value::Error("ERR: Argument must be a bulk string");
}

fn dbsize(_args: Vec<Value>, db: DB) -> Value {
    let set_len = db.lock().unwrap().set.len();
    let hset_len = db.lock().unwrap().hset.len();
    let total_len: i64 = (set_len + hset_len) as i64;
    Value::Num(total_len)
}

fn exists(args: Vec<Value>, db: DB) -> Value {
    if args.len() == 0 {
        return Value::Error("ERR: Wrong number of arguments provided");
    }

    let mut counter = 0i64;
    for val in args {
        if let Value::Bulk(key) = val {
            if db.lock().unwrap().set.contains_key(&key) {
                counter += 1;
            }
        }
    }
    Value::Num(counter)
}

fn hexists(args: Vec<Value>, db: DB) -> Value {
    if args.len() < 2 {
        return Value::Error("ERR: Wrong number of arguments provided");
    }

    let mut counter = 0i64;
    let Value::Bulk(key) = &args[0] else { return Value::Error(""); };
    let Value::Bulk(field) = &args[1] else { return Value::Error(""); };

    let map = match db.lock().unwrap().hset.get(key) {
        Some(m) => m.clone(),
        _ => return Value::Num(counter),
    };

    if map.contains_key(field) {
        counter += 1;
    }

    Value::Num(counter)
}

fn flushdb(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 0 {
        return Value::Null;
    }

    db.lock().unwrap().set.clear();
    db.lock().unwrap().hset.clear();

    std::fs::File::create("database.aof").unwrap();

    Value::Null
}

fn set(args: Vec<Value>, db: DB) -> Value {
    println!("{args:?}");
    if args.len() != 2 {
        return Value::Error("ERR: Wrong number of arguments provided");
    }

    let Value::Bulk(key) = &args[0] else {
        return Value::Error("ERR: Key must be a bulk string");
    };
    println!("{key}");

    let Value::Bulk(value) = &args[1] else {
        return Value::Error("ERR: Value must be a bulk string");
    };
    println!("{value}");

    db.lock().unwrap().set.insert(key.into(), value.into());
    println!("{:?}", db.lock().unwrap().set);

    Value::Str("OK")
}

fn get(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 1 {
        return Value::Error("ERR: Wrong number of arguments provided");
    }

    let Value::Bulk(key) = &args[0] else {
        return Value::Error("ERR: Key wasn't registered in database");
    };

    match db.lock().unwrap().set.get(key) {
        Some(s) => Value::Bulk(s.into()),
        _ => Value::Null,
    }
}

fn hset(args: Vec<Value>, db: DB) -> Value {
    if args.len() < 3 {
        return Value::Error("ERR: Wrong number of arguments provided");
    }

    let Value::Bulk(hash) = &args[0] else {
        return Value::Error("ERR: Incorrect definition for hash");
    };
    let Value::Bulk(key) = &args[1] else {
        return Value::Error("ERR: Incorrect definition for key");
    };
    let Value::Bulk(value) = &args[2] else {
        return Value::Error("ERR: Incorrect definition for value");
    };

    let map: HashMap<String, String> = HashMap::from([(key.into(), value.into())]);
    db.lock().unwrap().hset.insert(hash.into(), map);

    Value::Str("OK")
}

fn hget(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 2 {
        return Value::Error("ERR: Wrong number of arguments provided");
    }

    let Value::Bulk(hash) = &args[0] else {
        return Value::Error("ERR: Hash must be a bulk string");
    };
    let Value::Bulk(key) = &args[1] else {
        return Value::Error("ERR: Key must be a bulk string");
    };

    let map = match db.lock().unwrap().hset.get(hash) {
        Some(m) => m.clone(),
        _ => { return Value::Null; },
    };

    match map.get(key) {
        Some(s) => Value::Bulk(s.into()),
        _ => Value::Null,
    }
}

fn del(args: Vec<Value>, db: DB) -> Value {
    if args.len() == 0 {
        return Value::Error("ERR: No arguments were provided");
    }

    let mut counter = 0i64;
    for arg in args {
        if let Value::Bulk(key) = arg {
            match db.lock().unwrap().set.remove(&key) {
                Some(_) => counter += 1,
                _ => (),
            };
        } else {
            continue;
        }
    }
    Value::Num(counter)
}

fn hdel(args: Vec<Value>, db: DB) -> Value {
    if args.len() == 0 {
        return Value::Error("ERR: No arguments were provided");
    }
    if args.len() == 1 {
        return Value::Error("ERR: Wrong number of arguments provided");
    }

    let mut counter = 0i64;

    let Value::Bulk(hash) = &args[0] else {
        return Value::Error("ERR: Wrong definition for hash");
    };
    let Value::Bulk(key) = &args[1] else {
        return Value::Error("ERR: Wrong definition for key");
    };

    let mut hmap = match db.lock().unwrap().hset.get(hash) {
        Some(m) => m.clone(),
        _ => { return Value::Num(counter); },
    };
    match hmap.remove(key) {
        Some(_) => counter += 1,
        _ => (),
    };

    db.lock().unwrap().hset.insert(hash.into(), hmap.into());
    Value::Num(counter)
}

fn incr(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 1 {
        return Value::Error("ERR: Incorrect number of arguments");
    }

    let Value::Bulk(key) = &args[0] else {
        return Value::Error("ERR: Incorrect definition for key");
    };

    let mut value = 0i64;
    let mut err = "";
    
    db.lock().unwrap().set.entry(key.into())
        .and_modify(|val| {
            let v = match val.parse::<i64>() {
                Ok(n) => n,
                _ => {
                    err = "ERR: Value is not an integer or out of range";
                    return ();
                },
            };
            value = v + 1;
            *val = value.to_string()
        })
        .or_insert_with(|| {
            value += 1;
            value.to_string()
        });

    if err.len() != 0 {
        return Value::Error(err);
    }
    Value::Num(value)
}

fn incr_by(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 2 {
        return Value::Error("ERR: Wrong number of arguments");
    }

    let Value::Bulk(key) = &args[0] else {
        return Value::Error("ERR: Wrong definition for key");
    };

    let Value::Bulk(increment) = &args[1] else {
        return Value::Error("ERR: Value is not an integer or out of range");
    };

    let incr = match increment.parse::<i64>() {
        Ok(n) => n,
        _ => return Value::Error("ERR: Value is not an integer or out of range"),
    };

    let mut value = 0i64;
    let mut err = "";

    db.lock().unwrap().set.entry(key.into())
        .and_modify(|val| {
            let v = match val.parse::<i64>() {
                Ok(n) => n,
                _ => {
                    err = "ERR: Value is not an integer or out of range";
                    return ();
                },
            };
            value = v + incr;
            *val = value.to_string();
        }).or_insert_with(|| {
            value += incr;
            value.to_string()
        });

    if err.len() != 0 {
        return Value::Error(err);
    }
    Value::Num(value)
}

fn decr(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 1 {
        return Value::Error("ERR: Wrong number of arguments");
    }
    
    let Value::Bulk(key) = &args[0] else {
        return Value::Error("ERR: Wrong definition for key");
    };

    let mut value = 0i64;
    let mut err = "";
    
    db.lock().unwrap().set.entry(key.into())
        .and_modify(|val| {
            let v = match val.parse::<i64>() {
                Ok(n) => n,
                _ => {
                    err = "ERR: Value is not an integer or out of range";
                    return ();
                },
            };

            value = v - 1;
            *val = value.to_string()
        })
        .or_insert_with(|| {
            value -= 1;
            value.to_string()
        });

    if err.len() != 0 {
        return Value::Error(err);
    }
    Value::Num(value)
}

fn decr_by(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 2 {
        return Value::Error("ERR: Wrong number of arguments");
    }

    let Value::Bulk(key) = &args[0] else {
        return Value::Error("ERR: Incorrect definition for key");
    };

    let Value::Bulk(decrement) = &args[1] else {
        return Value::Error("ERR: Value is not an integer or out of range");
    };

    let decr = match decrement.parse::<i64>() {
        Ok(n) => n,
        _ => return Value::Error("ERR: Value is not an integer or out of range"),
    };

    let mut value = 0i64;
    let mut err = "";

    db.lock().unwrap().set.entry(key.into())
        .and_modify(|val| {
            let v = match val.parse::<i64>() {
                Ok(n) => n,
                _ => {
                    err = "ERR: Value is not an integer or out of range";
                    return ();
                },
            };
            value = v - decr;
            *val = value.to_string()
        })
        .or_insert_with(|| {
            value -= decr;
            value.to_string()
        });

    if err.len() != 0 {
        return Value::Error(err);
    }
    Value::Num(value)
}

fn multi(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 0 {
        return Value::Error("ERR: Wrong number of arguments");
    }

    db.lock().unwrap().set_transaction_mode(true);
    Value::Str("OK")
}

fn exec(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 0 {
        return Value::Error("ERR: Wrong number of arguments");
    }

    let transaction = db.lock().unwrap().multi.clone();
    let values: Vec<Value> = transaction
        .iter()
        .map(|(args, handler)| {
            handler(args.clone(), Arc::clone(&db))
        })
        .collect();
    db.lock().unwrap().multi.clear();

    Value::Array(values)
}

fn discard(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 0 {
        return Value::Error("ERR: Wrong number of arguments");
    }

    db.lock().unwrap().multi.clear();
    Value::Str("OK")
}
