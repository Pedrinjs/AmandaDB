use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::str::from_utf8;
use std::sync::{Arc, Mutex};

mod aof;
mod error;
mod handlers;
mod resp;
mod thread;

use aof::AOF;
use error::{new_error, Result};
use handlers::{handler::Handlers, types::Database};
use resp::{Resp, Value, Writer};
use thread::ThreadPool;

fn handle_read(value: Value, db: Arc<Mutex<Database>>) {
    let Value::Array(arr) = value else {
        println!("array only!");
        return;
    };

    let Value::Bulk(command) = &arr[0] else {
        println!("how?");
        return;
    };
    let args = &arr[1..];

    let mut handlers = Handlers::new();
    handlers.init();

    let cmd = command.to_uppercase();
    let handler = handlers.get(cmd.as_str()).unwrap();
    handler(args.to_vec(), db);
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379")?;
    let pool = ThreadPool::new(4);

    let db = Arc::new(Mutex::new(Database::new()));

    let aof = Arc::new(Mutex::new(AOF::new("database.aof".into())?));
    aof.lock().unwrap().read(handle_read, Arc::clone(&db))?;

    for stream in listener.incoming() {
        let stream = stream?;
        let aof = Arc::clone(&aof);
        let db = Arc::clone(&db);

        pool.execute(|| {
            match handle_request(stream, aof, db) {
                Ok(_) => (),
                Err(e) => println!("{e}"),
            };
        });
    };

    Ok(())
}

fn handle_request(mut stream: TcpStream, aof: Arc<Mutex<AOF>>, db: Arc<Mutex<Database>>) -> Result<()> {
    let mut buf = [0; 1024];
    stream.read(&mut buf)?;
    let request = from_utf8(&buf)?;

    let mut resp = Resp::new(request);
    let value = resp.read()?;

    /*let Value::Array(arr) = value.clone() else {
        return Err(new_error("Only arrays should be used."));
    };
    if arr.len() == 0 {
        return Err(new_error("An empty array was provided."));
    }

    let Value::Bulk(ref command) = arr[0] else {
        return Err(new_error("Can't get access to the command."));
    };

    let temp = command.to_uppercase();
    let cmd = temp.as_str();
    let args = &arr[1..];*/

    let mut handlers = Handlers::new();
    handlers.init();
    // let handler = handlers.get(cmd).unwrap();
    
    let mut writer = Writer::new(Box::new(stream));

    /*if cmd == "EXEC" || cmd == "DISCARD" {
        db.lock().unwrap().set_transaction_mode(false);
    }

    if db.lock().unwrap().is_transaction_mode() {
        db.lock().unwrap().multi.push((args.to_vec(), *handler));
        return writer.write(Value::Str("QUEUED".into()))
    }

    let command_list = vec!["SET", "HSET", "DEL", "HDEL", "INCR", "INCRBY", "DECR", "DECRBY"];
    if command_list.contains(&cmd) {
        aof.lock().unwrap().write(value)?;
    }*/

    // let result = handler(args.to_vec(), db);
    let result = handlers.match_handler(value, aof, db);
    if let Value::Error(err) = result {
        return Err(new_error(err));
    }
    writer.write(result)
}
