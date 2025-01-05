use std::io::Read;
use std::net::{TcpListener, TcpStream};
use std::sync::{Arc, Mutex};

use crate::aof::AOF;
use crate::error::{new_error, Result};
use crate::handlers::{handler::Handlers, types::Database};
use crate::resp::{reader::RESP, writer::Writer};
use crate::thread::ThreadPool;

pub struct Server {
    listener: TcpListener,
    pool: ThreadPool,
}

impl Server {
    pub fn new(port: usize) -> Result<Self> {
        let addr = format!("127.0.0.1:{port}");
        let listener = TcpListener::bind(addr)?;
        
        Ok(Self {
            listener,
            pool: ThreadPool::new(4),
        })
    }

    pub fn listen(&self, aof: Arc<Mutex<AOF>>, db: Arc<Mutex<Database>>) -> Result<()> {
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

fn handle_request(mut stream: TcpStream, aof: Arc<Mutex<AOF>>, db: Arc<Mutex<Database>>) -> Result<()> {
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
        if let Some(err) = result.is_error() {
            return Err(new_error(err));
        }
        writer.write(result)?;
    }
}
