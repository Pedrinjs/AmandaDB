use std::sync::{Arc, Mutex};

mod aof;
mod error;
mod handlers;
mod resp;
mod server;
mod thread;

use aof::AOF;
use error::Result;
use handlers::{handler::Handlers, types::Database};
use resp::value::Value;
use server::Server;

fn handle_read(value: Value, db: Arc<Mutex<Database>>) {
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
    let server = Server::new(6379)?;
    let aof = Arc::new(Mutex::new(AOF::new("database.aof")?));
    let db = Arc::new(Mutex::new(Database::new()));
    aof.lock().unwrap().read(handle_read, Arc::clone(&db))?;
    server.listen(aof, db)
}
