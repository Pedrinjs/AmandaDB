use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use crate::aof::AOF;
use crate::database::Database;
use crate::resp::Value;

type Aof = Arc<RwLock<AOF>>;
type DB = Arc<RwLock<Database>>;

type Handler = fn(Vec<Value>, DB) -> Value;

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

        let Value::BulkStr(command) = &arr[0] else {
            return Value::Error("ERR: The command must be a bulk string");
        };

        let cmd = command.to_uppercase();
        let handler = match self.get(&cmd) {
            Some(h) => h,
            None => return Value::Error("ERR: Command does not exist"),
        };

        if &cmd == "EXEC" || &cmd == "DISCARD" {
            db.write().unwrap().set_transaction_mode(false);
        }

        let args = &arr[1..];
        if db.read().unwrap().is_transaction_mode() {
            db.write().unwrap().multi_push(Value::BulkStr(command.into()), args.to_vec());
            return Value::Str("QUEUED");
        }
            
        let command_list = vec!["SET", "HSET", "DEL", "HDEL", "INCR", "INCRBY", "DECR", "DECRBY"];
        if command_list.contains(&cmd.as_str()) {
            if db.read().unwrap().is_execution_mode() {
                aof.write().unwrap().enqueue(input);
            } else {
                match aof.write().unwrap().write(input) {
                    Ok(_) => (),
                    Err(_) => return Value::Error("ERR: Failed to append to AOF"),
                };
            }
        }
        handler(args.to_vec(), db)
    }

    fn insert(&mut self, key: &'a str, handler: Handler) {
        self.handlers.insert(key, handler);
    }

    pub fn get(&self, key: &'a str) -> Option<&Handler> {
        self.handlers.get(key)
    }

    pub fn init(&mut self) {
        self.insert("COMMAND", command);

        self.insert("PING", ping);
        self.insert("ECHO", echo);
        self.insert("DBSIZE", dbsize);
        self.insert("HLEN", hlen);
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

fn command(_args: Vec<Value>, _db: DB) -> Value {
    Value::Str("OK")
}

fn ping(args: Vec<Value>, _db: DB) -> Value {
    if args.len() > 1 {
        return Value::Error("ERR: Wrong number of arguments for command");
    }

    if args.len() == 0 {
        return Value::Str("PONG");
    }
    if let Value::BulkStr(name) = &args[0] {
        return Value::BulkStr(name.into());
    }
    return Value::Str("PONG");
}

fn echo(args: Vec<Value>, _db: DB) -> Value {
    if args.len() != 1 {
        return Value::Error("ERR: Wrong number of arguments provided");
    }

    if let Value::BulkStr(name) = &args[0] {
        return Value::BulkStr(name.into());
    }
    return Value::Error("ERR: Argument must be a bulk string");
}

fn dbsize(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 0 {
        return Value::Error("ERR: Wrong number of arguments provided");
    }

    let set_len = db.read().unwrap().set_len();
    let hset_len = db.read().unwrap().hset_total_len();
    let total_len: i64 = (set_len + hset_len) as i64;
    Value::Num(total_len)
}

fn hlen(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 1 {
        return Value::Error("ERR: Wrong number of arguments provided");
    }

    if let Value::BulkStr(hash) = &args[0] {
        let len = db.read().unwrap().hset_len(hash);
        return Value::Num(len as i64)
    }
    Value::Error("ERR: Argument must be a bulk string")
}

fn exists(args: Vec<Value>, db: DB) -> Value {
    if args.len() == 0 {
        return Value::Error("ERR: Wrong number of arguments provided");
    }

    let mut counter = 0i64;
    args.iter().for_each(|val| {
        if let Value::BulkStr(key) = val {
            if db.read().unwrap().set_contains(&key) {
                counter += 1;
            }
        }
    });
    Value::Num(counter)
}

fn hexists(args: Vec<Value>, db: DB) -> Value {
    if args.len() == 0 || args.len() % 2 != 0 {
        return Value::Error("ERR: Wrong number of arguments provided");
    }

    let mut counter = 0i64;
    args.chunks(2).for_each(|arg| {
        if let [Value::BulkStr(hash), Value::BulkStr(key)] = arg {
            if db.read().unwrap().hset_contains(hash, key) {
                counter += 1;
            }
        }
    });
    Value::Num(counter)
}

fn flushdb(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 0 {
        return Value::Null;
    }

    db.write().unwrap().set_clear();
    db.write().unwrap().hset_clear();
    std::fs::File::create(db.read().unwrap().config().aof()).unwrap();
    Value::Null
}

fn set(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 2 {
        return Value::Error("ERR: Wrong number of arguments provided");
    }

    let Value::BulkStr(key) = &args[0] else {
        return Value::Error("ERR: Key must be a bulk string");
    };
    let Value::BulkStr(value) = &args[1] else {
        return Value::Error("ERR: Value must be a bulk string");
    };

    db.write().unwrap().set_push(key.into(), value.into());
    Value::Str("OK")
}

fn get(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 1 {
        return Value::Error("ERR: Wrong number of arguments provided");
    }

    let Value::BulkStr(key) = &args[0] else {
        return Value::Error("ERR: Key wasn't registered in database");
    };

    db.read().unwrap().set_get(key)
}

fn hset(args: Vec<Value>, db: DB) -> Value {
    if args.len() < 3 {
        return Value::Error("ERR: Wrong number of arguments provided");
    }

    let Value::BulkStr(hash) = &args[0] else {
        return Value::Error("ERR: Incorrect definition for hash");
    };
    let Value::BulkStr(key) = &args[1] else {
        return Value::Error("ERR: Incorrect definition for key");
    };
    let Value::BulkStr(value) = &args[2] else {
        return Value::Error("ERR: Incorrect definition for value");
    };

    db.write().unwrap().hset_push(hash.into(), key.into(), value.into());
    Value::Str("OK")
}

fn hget(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 2 {
        return Value::Error("ERR: Wrong number of arguments provided");
    }

    let Value::BulkStr(hash) = &args[0] else {
        return Value::Error("ERR: Hash must be a bulk string");
    };
    let Value::BulkStr(key) = &args[1] else {
        return Value::Error("ERR: Key must be a bulk string");
    };

    db.read().unwrap().hset_get(hash, key)
}

fn del(args: Vec<Value>, db: DB) -> Value {
    if args.len() == 0 {
        return Value::Error("ERR: No arguments were provided");
    }

    let mut counter = 0i64;
    args.iter().for_each(|arg| {
        if let Value::BulkStr(key) = arg {
            if db.write().unwrap().set_remove(&key) {
                counter += 1;
            }
        }
    });
    Value::Num(counter)
}

fn hdel(args: Vec<Value>, db: DB) -> Value {
    if args.len() == 0 || args.len() % 2 != 0 {
        return Value::Error("ERR: Wrong number of arguments provided");
    }

    let mut counter = 0i64;
    args.chunks(2).for_each(|arg| {
        if let [Value::BulkStr(hash), Value::BulkStr(key)] = arg {
            if db.write().unwrap().hset_remove(hash, key) {
                counter += 1;
            }
        }
    });

    Value::Num(counter)
}

fn incr(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 1 {
        return Value::Error("ERR: Incorrect number of arguments");
    }

    let Value::BulkStr(key) = &args[0] else {
        return Value::Error("ERR: Incorrect definition for key");
    };

    db.write().unwrap().set_incr(key.into(), 1)
}

fn incr_by(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 2 {
        return Value::Error("ERR: Wrong number of arguments");
    }

    let Value::BulkStr(key) = &args[0] else {
        return Value::Error("ERR: Wrong definition for key");
    };

    let Value::BulkStr(increment) = &args[1] else {
        return Value::Error("ERR: Value is not an integer or out of range");
    };

    let incr = match increment.parse::<i64>() {
        Ok(n) => n,
        _ => return Value::Error("ERR: Value is not an integer or out of range"),
    };

    db.write().unwrap().set_incr(key.into(), incr)
}

fn decr(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 1 {
        return Value::Error("ERR: Wrong number of arguments");
    }
    let Value::BulkStr(key) = &args[0] else {
        return Value::Error("ERR: Wrong definition for key");
    };

    db.write().unwrap().set_incr(key.into(), -1)
}

fn decr_by(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 2 {
        return Value::Error("ERR: Wrong number of arguments");
    }

    let Value::BulkStr(key) = &args[0] else {
        return Value::Error("ERR: Incorrect definition for key");
    };
    let Value::BulkStr(decrement) = &args[1] else {
        return Value::Error("ERR: Value is not an integer or out of range");
    };

    let decr = match decrement.parse::<i64>() {
        Ok(n) => n,
        _ => return Value::Error("ERR: Value is not an integer or out of range"),
    };

    db.write().unwrap().set_incr(key.into(), -decr)
}

fn multi(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 0 {
        return Value::Error("ERR: Wrong number of arguments");
    }
    db.write().unwrap().set_transaction_mode(true);
    Value::Str("OK")
}

fn exec(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 0 {
        return Value::Error("ERR: Wrong number of arguments");
    }

    let config = db.read().unwrap().config();
    let aof = Arc::new(RwLock::new(match AOF::new(config) {
        Ok(file) => file,
        Err(_) => return Value::Error("ERR: Failed to access AOF"),
    }));

    let mut handlers = Handlers::new();
    handlers.init();

    let transaction = db.read().unwrap().multi_get();
    db.write().unwrap().set_execution_mode(true);
    let copy = db.read().unwrap().create_database_copy();

    let mut values: Vec<Value> = Vec::new();
    for (cmd, args) in transaction.into_iter() {
        let mut input: Vec<Value> = Vec::new();
        input.push(cmd);
        input.extend(args);
        let command = Value::Array(input);

        let value = handlers.match_handler(command, aof.clone(), db.clone());
        if let Value::Error(_) = value {
            db.write().unwrap().database_revert(copy);
            break;
        }
        values.push(value)
    }
    db.write().unwrap().multi_clear();
    db.write().unwrap().set_execution_mode(false);

    match Arc::clone(&aof).write().unwrap().write_queued() {
        Ok(_) => Value::Array(values),
        Err(_) => Value::Error("ERR: Failed to write to AOF"),
    }
}

fn discard(args: Vec<Value>, db: DB) -> Value {
    if args.len() != 0 {
        return Value::Error("ERR: Wrong number of arguments");
    }
    db.write().unwrap().multi_clear();
    Value::Str("OK")
}
