use std::sync::{Arc, RwLock};

mod aof;
mod config;
mod error;
mod handlers;
mod resp;
mod server;
mod thread;

use aof::AOF;
use config::Config;
use error::Result;
use handlers::{handler::Handlers, types::Database};
use resp::value::Value;
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
    let config = Config::default();

    let server = Server::new(config)?;
    let aof = Arc::new(RwLock::new(AOF::new(config)?));
    let db = Arc::new(RwLock::new(Database::new(config)));
    aof.write().unwrap().read(handle_read, Arc::clone(&db))?;

    server.listen(aof, db)
}
