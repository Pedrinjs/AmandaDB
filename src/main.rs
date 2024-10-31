use std::io::prelude::*;
use std::net::{TcpListener, TcpStream};
use std::str::from_utf8;
use std::sync::{Arc, Mutex};

mod aof;
mod error;
mod handler;
mod resp;
mod thread;

use aof::AOF;
use error::{new_error, Result};
use handler::Handlers;
use resp::{Resp, Value, Writer};
use thread::ThreadPool;

fn handle_read(value: Value) {
    let Value::Array(arr) = value else {
        println!("array only!");
        return;
    };

    let Value::Bulk(ref command) = arr[0] else {
        println!("how?");
        return;
    };
    let args = &arr[1..];

    let mut handlers = Handlers::new();
    handlers.init();

    let handler = handlers.get(command.to_uppercase());
    handler(args.to_vec());
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379").unwrap();
    let tpool = ThreadPool::new(4);

    let aof = Arc::new(Mutex::new(AOF::new("database.aof".into())?));
    aof.lock().unwrap().read(handle_read)?;

    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let aof = Arc::clone(&aof);

        tpool.execute(|| {
            match handle_request(stream, aof) {
                Ok(_) => (),
                Err(e) => println!("{e}"),
            };
        });
    };

    Ok(())
}

fn handle_request(mut stream: TcpStream, aof: Arc<Mutex<AOF>>) -> Result<()> {
    let mut buf = [0; 1024];
    stream.read(&mut buf)?;
    let request = from_utf8(&buf)?;

    let mut resp = Resp::new(request);
    let value = resp.read()?;

    let Value::Array(arr) = value.clone() else {
        return Err(new_error("Only arrays should be used."));
    };
    if arr.len() == 0 {
        return Err(new_error("An empty array was provided."));
    }

    let Value::Bulk(ref command) = arr[0] else {
        return Err(new_error("Can't get access to the command."));
    };
    let args = &arr[1..];

    let mut writer = Writer::new(stream);

    let mut handlers = Handlers::new();
    handlers.init();
    let handler = handlers.get(command.to_uppercase());
        
    if &command.to_uppercase() == "SET" || &command.to_uppercase() == "HSET" {
        aof.lock().unwrap().write(value)?;
    }

    let result = handler(args.to_vec());
    writer.write(result)?;

    Ok(())
}
