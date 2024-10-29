use std::collections::BTreeMap;
use std::sync::{Arc, LazyLock, RwLock};

use crate::resp::Value;

static SET: LazyLock<Arc<RwLock<BTreeMap<String, String>>>> = LazyLock::new(|| {
    Arc::new(RwLock::new(BTreeMap::new()))
});

static HSET: LazyLock<Arc<RwLock<BTreeMap<String, BTreeMap<String, String>>>>> = LazyLock::new(|| {
    Arc::new(RwLock::new(BTreeMap::new()))
});

type Handler = fn(Vec<Value>) -> Value;

pub struct Handlers {
    handlers: BTreeMap<String, Handler>
}

impl Handlers {
    pub fn new() -> Self {
        Handlers{
            handlers: BTreeMap::new(),
        }
    }

    pub fn init(&mut self) {
        self.handlers.insert("PING".into(), ping);
        self.handlers.insert("ECHO".into(), echo);

        self.handlers.insert("DBSIZE".into(), dbsize);
        self.handlers.insert("EXISTS".into(), exists);
        self.handlers.insert("FLUSHDB".into(), flushdb);

        self.handlers.insert("SET".into(), set);
        self.handlers.insert("HSET".into(), hset);

        self.handlers.insert("GET".into(), get);
        self.handlers.insert("HGET".into(), hget);

        // self.handlers.insert("DEL".into(), del);
        // self.handlers.insert("HDEL".into(), hdel);
    }

    pub fn get(&self, key: String) -> &Handler {
        self.handlers.get(&key).unwrap()
    }
}

fn ping(args: Vec<Value>) -> Value {
    let default: String = "PONG".into();
    
    if args.len() > 1 {
        return Value::Error("ERR: Wrong number of arguments for command".into());
    }

    if args.len() == 0 {
        return Value::Str(default);
    }

    let Value::Bulk(name) = &args[0] else {
        return Value::Str(default);
    };
    return Value::Bulk(name.into());
}

fn echo(args: Vec<Value>) -> Value {
    if args.len() != 1 {
        return Value::Error("ERR: Wrong number of arguments for command".into());
    }

    let Value::Bulk(name) = &args[0] else {
        return Value::Error("ERR: Only bulk is acceptable".into());
    };

    return Value::Bulk(name.into());
}

fn dbsize(_args: Vec<Value>) -> Value {
    let set_len = SET.read().unwrap().len();
    let hset_len = HSET.read().unwrap().len();

    let total_len: i64 = (set_len + hset_len) as i64;
    Value::Num(total_len)
}

fn exists(args: Vec<Value>) -> Value {
    if args.len() == 0 {
        return Value::Error("ERR: Only bulk is acceptable".into());
    }

    let mut counter: i64 = 0;
    println!("{args:?}");

    for val in args {
        println!("{val:?}");
        if let Value::Bulk(key) = val {
            if SET.read().unwrap().contains_key(&key) {
                counter += 1;
            }
            
            if HSET.read().unwrap().contains_key(&key) {
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

    SET.write().unwrap().clear();
    HSET.write().unwrap().clear();

    std::fs::File::create("database.aof").unwrap();

    Value::Null
}

fn set(args: Vec<Value>) -> Value {
    let wrong_err = Value::Error("ERR: Wrong number of arguments for command".into());
    let args_err = Value::Error("ERR: Incorrect definition for key or value".into());
        
    if args.len() != 2 {
        return wrong_err;
    }

    let Value::Bulk(key) = &args[0] else { return args_err; };
    let Value::Bulk(value) = &args[1] else { return args_err; };

    SET.write().unwrap().insert(key.into(), value.into());

    Value::Str("OK".into())
}

fn get(args: Vec<Value>) -> Value {
    let wrong_err = Value::Error("ERR: Wrong number of arguments for 'get' command".into());
    let args_err = Value::Error("ERR: Key wasn't registered in database".into());
        
    if args.len() != 1 {
        return wrong_err;
    }

    let Value::Bulk(key) = &args[0] else { return args_err; };

    match SET.read().unwrap().get(key) {
        Some(s) => Value::Bulk(s.into()),
        None => Value::Null,
    }
}

fn hset(args: Vec<Value>) -> Value {
    let wrong_err = Value::Error("ERR: Wrong number of arguments for 'hset' command".into());
    let args_err = Value::Error("ERR: Incorrect definition for hash, key or value".into());

    if args.len() != 3 {
        return wrong_err;
    }

    let Value::Bulk(hash) = &args[0] else { return args_err; };
    let Value::Bulk(key) = &args[1] else { return args_err; };
    let Value::Bulk(value) = &args[2] else { return args_err; };

    let map: BTreeMap<String, String> = BTreeMap::from([(key.into(), value.into())]);
    HSET.write().unwrap().insert(hash.into(), map);

    Value::Str("OK".into())
}

fn hget(args: Vec<Value>) -> Value {
    let wrong_err = Value::Error("ERR: Wrong number of arguments for 'hget' command".into());
    let args_err = Value::Error("ERR: Hash or key weren't registered in database".into());

    if args.len() != 2 {
        return wrong_err;
    }

    let Value::Bulk(hash) = &args[0] else { return args_err; };
    let Value::Bulk(key) = &args[1] else { return args_err; };

    let map = match HSET.read().unwrap().get(hash) {
        Some(m) => m.clone(),
        None => { return Value::Null; },
    };

    match map.get(key) {
        Some(s) => Value::Bulk(s.into()),
        None => Value::Null,
    }
}

/*fn del(args: Vec<Value>) -> Value {
    if args.len() == 0 {
        return Value::Error("ERR: No arguments were provided".into());
    }

    let mut counter: i64 = 0;

    for arg in args {
        if let Value::Bulk(key) = arg {
            match SET.write().unwrap().remove(&key) {
                Some(_) => counter += 1,
                _ => (),
            };
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

    let Value::Bulk(hash) = &args[0] else { return Value::Num(counter); };
    let keys = args[1..].to_vec();

    let map = match HSET.read().unwrap().get(hash) {
        Some(m) => m.clone(),
        None => return Value::Num(counter),
    };

    for key in keys {
        if let Value::Bulk(key) = key {
            if !map.contains_key(&key) {
                continue;
            }

            match HSET.write().unwrap().remove(&key) {
                Some(_) => counter += 1,
                _ => (),
            };
        }
    }

    counter += 1;

    Value::Num(counter)
}*/
