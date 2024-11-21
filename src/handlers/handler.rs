use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use super::types::{Database, Handler};
use crate::resp::Value;

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

    pub fn init(&mut self) {
        self.handlers.insert("PING", ping);
        self.handlers.insert("ECHO", echo);

        self.handlers.insert("DBSIZE", dbsize);
        self.handlers.insert("FLUSHDB", flushdb);

        self.handlers.insert("EXISTS", exists);
        self.handlers.insert("HEXISTS", hexists);

        self.handlers.insert("SET", set);
        self.handlers.insert("HSET", hset);

        self.handlers.insert("GET", get);
        self.handlers.insert("HGET", hget);

        self.handlers.insert("DEL", del);
        self.handlers.insert("HDEL", hdel);

        self.handlers.insert("INCR", incr);
        self.handlers.insert("INCRBY", incr_by);

        self.handlers.insert("DECR", decr);
        self.handlers.insert("DECRBY", decr_by);

        self.handlers.insert("MULTI", multi);
        self.handlers.insert("EXEC", exec);
        self.handlers.insert("DISCARD", discard);
    }

    pub fn get(&self, key: &str) -> &Handler {
        self.handlers.get(key).unwrap()
    }
}

fn ping(args: Vec<Value>, _db: DB) -> Value {
    if args.len() > 1 {
        return Value::Error("ERR: Wrong number of arguments for command".into());
    }

    if args.len() == 0 {
        return Value::Str("PONG".into());
    }

    let Value::Bulk(name) = &args[0] else {
        return Value::Str("PONG".into());
    };
    return Value::Bulk(name.into());
}

fn echo(args: Vec<Value>, _db: DB) -> Value {
    if args.len() != 1 {
        return Value::Error("ERR: Wrong number of arguments provided".into());
    }

    let Value::Bulk(name) = &args[0] else {
        return Value::Error("ERR: Argument must be a bulk string".into());
    };

    return Value::Bulk(name.into());
}

fn dbsize(_args: Vec<Value>, db: DB) -> Value {
    let set_len = db.lock().unwrap().set.len();
    let hset_len = db.lock().unwrap().hset.len();
    let total_len: i64 = (set_len + hset_len) as i64;
    Value::Num(total_len)
}

fn exists(args: Vec<Value>, db: DB) -> Value {
    if args.len() == 0 {
        return Value::Error("ERR: Wrong number of arguments provided".into());
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
    if args.len() == 0 {
        return Value::Error("ERR: Wrong number of arguments provided".into());
    }

    let mut counter = 0i64;
    for val in args {
        if let Value::Bulk(key) = val {
            if db.lock().unwrap().hset.contains_key(&key) {
                counter += 1;
            }
        }
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
        return Value::Error("ERR: Wrong number of arguments provided".into());
    }

    let Value::Bulk(key) = &args[0] else {
        return Value::Error("ERR: Key must be a bulk string".into());
    };
    println!("{key}");

    let Value::Bulk(value) = &args[1] else {
        return Value::Error("ERR: Value must be a bulk string".into());
    };
    println!("{value}");

    db.lock().unwrap().set.insert(key.into(), value.into());
    println!("{:?}", db.lock().unwrap().set);

    Value::Str("OK".into())
}

fn get(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 1 {
        return Value::Error("ERR: Wrong number of arguments provided".into());
    }

    let Value::Bulk(key) = &args[0] else {
        return Value::Error("ERR: Key wasn't registered in database".into());
    };

    match db.lock().unwrap().set.get(key) {
        Some(s) => Value::Bulk(s.into()),
        _ => Value::Null,
    }
}

fn hset(args: Vec<Value>, db: DB) -> Value {
    if args.len() < 3 {
        return Value::Error("ERR: Wrong number of arguments provided".into());
    }

    let Value::Bulk(hash) = &args[0] else {
        return Value::Error("ERR: Incorrect definition for hash".into());
    };
    let Value::Bulk(key) = &args[1] else {
        return Value::Error("ERR: Incorrect definition for key".into());
    };
    let Value::Bulk(value) = &args[2] else {
        return Value::Error("ERR: Incorrect definition for value".into());
    };

    let map: HashMap<String, String> = HashMap::from([(key.into(), value.into())]);
    db.lock().unwrap().hset.insert(hash.into(), map);

    Value::Str("OK".into())
}

fn hget(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 2 {
        return Value::Error("ERR: Wrong number of arguments provided".into());
    }

    let Value::Bulk(hash) = &args[0] else {
        return Value::Error("ERR: Hash must be a bulk string".into());
    };
    let Value::Bulk(key) = &args[1] else {
        return Value::Error("ERR: Key must be a bulk string".into());
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
        return Value::Error("ERR: No arguments were provided".into());
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
        return Value::Error("ERR: No arguments were provided".into());
    }
    if args.len() == 1 {
        return Value::Error("ERR: Wrong number of arguments provided".into());
    }

    let mut counter = 0i64;

    let Value::Bulk(hash) = &args[0] else {
        return Value::Error("ERR: Wrong definition for hash".into());
    };
    let Value::Bulk(key) = &args[1] else {
        return Value::Error("ERR: Wrong definition for key".into());
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
        return Value::Error("ERR: Incorrect number of arguments".into());
    }

    let Value::Bulk(key) = &args[0] else {
        return Value::Error("ERR: Incorrect definition for key".into());
    };

    let mut value = 0i64;
    let mut err = String::new();
    
    db.lock().unwrap().set.entry(key.into())
        .and_modify(|val| {
            let v = match val.parse::<i64>() {
                Ok(n) => n,
                _ => {
                    err = "ERR: Value is not an integer or out of range".into();
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
        return Value::Error("ERR: Wrong number of arguments".into());
    }

    let Value::Bulk(key) = &args[0] else {
        return Value::Error("ERR: Wrong definition for key".into());
    };

    let Value::Bulk(increment) = &args[1] else {
        return Value::Error("ERR: Value is not an integer or out of range".into());
    };

    let incr = match increment.parse::<i64>() {
        Ok(n) => n,
        _ => return Value::Error("ERR: Value is not an integer or out of range".into()),
    };

    let mut value = 0i64;
    let mut err = String::new();

    db.lock().unwrap().set.entry(key.into())
        .and_modify(|val| {
            let v = match val.parse::<i64>() {
                Ok(n) => n,
                _ => {
                    err = "ERR: Value is not an integer or out of range".to_string();
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
        return Value::Error("ERR: Wrong number of arguments".into());
    }
    
    let Value::Bulk(key) = &args[0] else {
        return Value::Error("ERR: Wrong definition for key".into());
    };

    let mut value = 0i64;
    let mut err = String::new();
    
    db.lock().unwrap().set.entry(key.into())
        .and_modify(|val| {
            let v = match val.parse::<i64>() {
                Ok(n) => n,
                _ => {
                    err = "ERR: Value is not an integer or out of range".into();
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
        return Value::Error("ERR: Wrong number of arguments".into());
    }

    let Value::Bulk(key) = &args[0] else {
        return Value::Error("ERR: Incorrect definition for key".into());
    };

    let Value::Bulk(decrement) = &args[1] else {
        return Value::Error("ERR: Value is not an integer or out of range".into());
    };

    let decr = match decrement.parse::<i64>() {
        Ok(n) => n,
        _ => return Value::Error("ERR: Value is not an integer or out of range".into()),
    };

    let mut value = 0i64;
    let mut err = String::new();

    db.lock().unwrap().set.entry(key.into())
        .and_modify(|val| {
            let v = match val.parse::<i64>() {
                Ok(n) => n,
                _ => {
                    err = "ERR: Value is not an integer or out of range".into();
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
        return Value::Error("ERR: Wrong number of arguments".into());
    }

    db.lock().unwrap().set_transaction_mode(true);
    Value::Str("OK".into())
}

fn exec(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 0 {
        return Value::Error("ERR: Wrong number of arguments".into());
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
        return Value::Error("ERR: Wrong number of arguments".into());
    }

    db.lock().unwrap().multi.clear();
    Value::Str("OK".into())
}
