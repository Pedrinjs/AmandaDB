use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, RwLock};

use crate::aof::AOF;
use crate::config::Config;
use crate::error::{new_error, Result};
use crate::handlers::Handlers;
use crate::database::Database;
use crate::resp::{RESP, Writer};
use crate::thread::ThreadPool;

pub struct Server {
    listener: TcpListener,
    pool: ThreadPool,
}

impl Server {
    pub fn new(config: Config) -> Result<Self> {
        let listener = TcpListener::bind(("127.0.0.1", config.port()))?;
        let pool = ThreadPool::new(config.threads());
        
        Ok(Self {
            listener,
            pool,
        })
    }

    pub fn listen(&self, aof: Arc<RwLock<AOF>>, db: Arc<RwLock<Database>>) -> Result<()> {
        for stream in self.listener.incoming() {
            let stream = stream?;
            let aof = Arc::clone(&aof);
            let db = Arc::clone(&db);

            self.pool.execute(|| {
                if let Err(e) = handle_request(stream, aof, db) {
                    eprintln!("{e}");
                }
            });
        }
        println!("shutting down");
        Ok(())
    }
}

fn handle_request(mut stream: TcpStream, aof: Arc<RwLock<AOF>>, db: Arc<RwLock<Database>>) -> Result<()> {
    let mut buffer = [0; 1024];

    loop {
        if stream.read(&mut buffer)? == 0 {
            return Ok(());
        }

        let request = std::str::from_utf8(&buffer)?;
        let mut resp = RESP::new(request);
        let value = resp.read()?;

        let mut handlers = Handlers::new();
        handlers.init();
        let mut writer = Writer::new(Box::new(stream.try_clone()?));

        let result = handlers.match_handler(value, aof.clone(), db.clone());
        match result.is_error() {
            Some(err) => return Err(new_error(err)),
            _ => writer.write(result)?,
        };
    }
}
