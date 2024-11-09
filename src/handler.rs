use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex};

use crate::resp::Value;

static SET: LazyLock<Arc<Mutex<HashMap<String, String>>>> = LazyLock::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

static HSET: LazyLock<Arc<Mutex<HashMap<String, HashMap<String, String>>>>> = LazyLock::new(|| {
    Arc::new(Mutex::new(HashMap::new()))
});

type Handler = fn(Vec<Value>) -> Value;

pub struct Handlers {
    handlers: HashMap<String, Handler>
}

impl Handlers {
    pub fn new() -> Self {
        Handlers{
            handlers: HashMap::new(),
        }
    }

    pub fn init(&mut self) {
        self.handlers.insert("PING".into(), ping);
        self.handlers.insert("ECHO".into(), echo);

        self.handlers.insert("DBSIZE".into(), dbsize);
        self.handlers.insert("FLUSHDB".into(), flushdb);

        self.handlers.insert("EXISTS".into(), exists);
        self.handlers.insert("HEXISTS".into(), hexists);

        self.handlers.insert("SET".into(), set);
        self.handlers.insert("HSET".into(), hset);

        self.handlers.insert("GET".into(), get);
        self.handlers.insert("HGET".into(), hget);

        self.handlers.insert("DEL".into(), del);
        self.handlers.insert("HDEL".into(), hdel);

        self.handlers.insert("INCR".into(), incr);
        self.handlers.insert("DECR".into(), decr);
    }

    pub fn get(&self, key: String) -> &Handler {
        self.handlers.get(&key).unwrap()
    }
}

fn ping(args: Vec<Value>) -> Value {
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

fn echo(args: Vec<Value>) -> Value {
    if args.len() != 1 {
        return Value::Error("ERR: Wrong number of arguments provided".into());
    }

    let Value::Bulk(name) = &args[0] else {
        return Value::Error("ERR: Argument must be a bulk string".into());
    };

    return Value::Bulk(name.into());
}

fn dbsize(_args: Vec<Value>) -> Value {
    let set_len = SET.lock().unwrap().len();
    let hset_len = HSET.lock().unwrap().len();
    let total_len: i64 = (set_len + hset_len) as i64;
    Value::Num(total_len)
}

fn exists(args: Vec<Value>) -> Value {
    if args.len() == 0 {
        return Value::Error("ERR: Wrong number of arguments provided".into());
    }

    let mut counter: i64 = 0;
    for val in args {
        if let Value::Bulk(key) = val {
            if SET.lock().unwrap().contains_key(&key) {
                counter += 1;
            }
        }
    }
    Value::Num(counter)
}

fn hexists(args: Vec<Value>) -> Value {
    if args.len() == 0 {
        return Value::Error("ERR: Wrong number of arguments provided".into());
    }

    let mut counter: i64 = 0;
    for val in args {
        if let Value::Bulk(key) = val {
            if HSET.lock().unwrap().contains_key(&key) {
                counter += 1;
            }
        }
    }
    Value::Num(counter)
}

fn flushdb(args: Vec<Value>) -> Value {
    if args.len() != 0 {
        return Value::Null;
    }

    SET.lock().unwrap().clear();
    HSET.lock().unwrap().clear();

    std::fs::File::create("database.aof").unwrap();

    Value::Null
}

fn set(args: Vec<Value>) -> Value {
    if args.len() != 2 {
        return Value::Error("ERR: Wrong number of arguments provided".into());
    }

    let Value::Bulk(key) = &args[0] else {
        return Value::Error("ERR: Key must be a bulk string".into());
    };
    let Value::Bulk(value) = &args[1] else {
        return Value::Error("ERR: Value must be a bulk string".into());
    };

    SET.lock().unwrap().insert(key.into(), value.into());

    Value::Str("OK".into())
}

fn get(args: Vec<Value>) -> Value {
    if args.len() != 1 {
        return Value::Error("ERR: Wrong number of arguments provided".into());
    }

    let Value::Bulk(key) = &args[0] else {
        return Value::Error("ERR: Key wasn't registered in database".into());
    };

    match SET.lock().unwrap().get(key) {
        Some(s) => Value::Bulk(s.into()),
        _ => Value::Null,
    }
}

fn hset(args: Vec<Value>) -> Value {
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
    HSET.lock().unwrap().insert(hash.into(), map);

    Value::Str("OK".into())
}

fn hget(args: Vec<Value>) -> Value {
    if args.len() != 2 {
        return Value::Error("ERR: Wrong number of arguments provided".into());
    }

    let Value::Bulk(hash) = &args[0] else {
        return Value::Error("ERR: Hash must be a bulk string".into());
    };
    let Value::Bulk(key) = &args[1] else {
        return Value::Error("ERR: Key must be a bulk string".into());
    };

    let map = match HSET.lock().unwrap().get(hash) {
        Some(m) => m.clone(),
        _ => { return Value::Null; },
    };

    match map.get(key) {
        Some(s) => Value::Bulk(s.into()),
        _ => Value::Null,
    }
}

fn del(args: Vec<Value>) -> Value {
    if args.len() == 0 {
        return Value::Error("ERR: No arguments were provided".into());
    }

    let mut counter: i64 = 0;
    for arg in args {
        if let Value::Bulk(key) = arg {
            match SET.lock().unwrap().remove(&key) {
                Some(_) => counter += 1,
                _ => (),
            };
        } else {
            continue;
        }
    }
    Value::Num(counter)
}

fn hdel(args: Vec<Value>) -> Value {
    if args.len() == 0 {
        return Value::Error("ERR: No arguments were provided".into());
    }
    if args.len() == 1 {
        return Value::Error("ERR: Wrong number of arguments provided".into());
    }

    let mut counter: i64 = 0;

    let Value::Bulk(hash) = &args[0] else {
        return Value::Error("ERR: Wrong definition for hash".into());
    };
    let Value::Bulk(key) = &args[1] else {
        return Value::Error("ERR: Wrong definition for key".into());
    };

    let mut hmap = match HSET.lock().unwrap().get(hash) {
        Some(m) => m.clone(),
        _ => { return Value::Num(counter); },
    };

    match hmap.remove(key) {
        Some(_) => counter += 1,
        _ => (),
    };

    HSET.lock().unwrap().insert(hash.into(), hmap.into());

    Value::Num(counter)
}

fn incr(args: Vec<Value>) -> Value {
    if args.len() != 1 {
        return Value::Error("ERR: Incorrect number of arguments".into());
    }

    let Value::Bulk(key) = &args[0] else {
        return Value::Error("ERR: Key must be a bulk string".into());
    };

    let mut value = 0;
    let mut err = String::new();
    
    SET.lock().unwrap().entry(key.into())
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

fn decr(args: Vec<Value>) -> Value {
    if args.len() != 1 {
        return Value::Error("ERR: Wrong number of arguments".into());
    }
    
    let Value::Bulk(key) = &args[0] else {
        return Value::Error("ERR: Wrong definition for key".into());
    };

    let mut value = 0;
    let mut err = String::new();
    
    SET.lock().unwrap().entry(key.into())
        .and_modify(|val| {
            let v = match val.parse::<i64>() {
                Ok(n) => n,
                _ => { err = "ERR: Value is not an integer or out of range".into(); return (); },
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
