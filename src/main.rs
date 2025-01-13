use std::sync::{Arc, RwLock};

mod aof;
mod config;
mod database;
mod error;
mod handlers;
mod resp;
mod server;
mod thread;

use aof::AOF;
use config::Config;
use error::Result;
use handlers::Handlers;
use database::Database;
use resp::Value;
use server::Server;

fn handle_read(value: Value, db: Arc<RwLock<Database>>) {
    let Value::Array(arr) = value else {
        eprintln!("array only!");
        return;
    };

    let Value::BulkStr(command) = &arr[0] else {
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
    let mut args = std::env::args();
    args.next();

    let config = match args.next() {
        Some(path) => Config::read_from_file(&path)?,
        None => Config::default(),
    };

    let server = Server::new(config.clone())?;
    let aof = Arc::new(RwLock::new(AOF::new(config.clone())?));
    let db = Arc::new(RwLock::new(Database::new(config.clone())));
    aof.write().unwrap().read(handle_read, Arc::clone(&db))?;

    server.listen(aof, db)
}
