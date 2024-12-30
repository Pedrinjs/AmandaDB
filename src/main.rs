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
use resp::{reader::RESP, value::Value, writer::Writer};
use thread::ThreadPool;

fn handle_read(value: Value, db: Arc<Mutex<Database>>) {
    let Value::Array(arr) = value else {
        eprintln!("array only!");
        return;
    };

    let Value::Bulk(command) = &arr[0] else {
        eprintln!("only array of bulk strings");
        return;
    };
    let args = &arr[1..];

    let mut handlers = Handlers::new();
    handlers.init();

    let cmd = command.to_uppercase();
    let handler = handlers.get(cmd.as_str()).unwrap();
    handler(args.to_vec(), Arc::clone(&db));
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:6379")?;
    let pool = ThreadPool::new(4);

    let aof = Arc::new(Mutex::new(AOF::new("database.aof")?));
    let db = Arc::new(Mutex::new(Database::new()));
    aof.lock().unwrap().read(handle_read, Arc::clone(&db))?;

    for stream in listener.incoming() {
        let stream = stream?;
        let aof = Arc::clone(&aof);
        let db = Arc::clone(&db);

        pool.execute(|| {
            if let Err(e) = handle_request(stream, aof, db) {
                eprintln!("{e}");
            }
        });
    };

    println!("shutting down");
    Ok(())
}

fn handle_request(mut stream: TcpStream, aof: Arc<Mutex<AOF>>, db: Arc<Mutex<Database>>) -> Result<()> {
    let mut buf = [0; 1024];

    loop {
        if stream.read(&mut buf)? == 0 {
            // it only returns OK because I am assuming the
            // client closed the connection by it's own
            return Ok(());
        }

        let request = from_utf8(&buf)?;
        let mut resp = RESP::new(request);
        let value = resp.read()?;

        let mut handlers = Handlers::new();
        handlers.init();
        let mut writer = Writer::new(Box::new(stream.try_clone()?));

        let result = handlers.match_handler(value, aof.clone(), db.clone());
        if let Value::Error(err) = result {
            return Err(new_error(err));
        }
        writer.write(result)?
    }
}
