#[derive(Copy, Clone)]
pub struct Config {
    aof: &'static str,
    port: usize,
    threads: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            aof: "database.aof",
            port: 6379,
            threads: 4,
        }
    }
}

impl Config {
    pub fn aof(&self) -> &str {
        self.aof
    }

    pub fn port(&self) -> usize {
        self.port
    }

    pub fn threads(&self) -> usize {
        self.threads
    }
}
