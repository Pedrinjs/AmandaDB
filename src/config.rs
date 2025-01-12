use crate::error::{new_error, Result};

#[derive(Clone)]
pub struct Config {
    aof: String,
    port: usize,
    threads: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            aof: "database.aof".into(),
            port: 6379,
            threads: 4,
        }
    }
}

impl Config {
    pub fn read_from_file(path: &str) -> Result<Config> {
        let mut config = Config::default();

        let contents = std::fs::read_to_string(path)?;
        for line in contents.lines() {
            let kv: Vec<&str> = line.split('=').collect();
            match (kv[0].trim(), kv[1].trim()) {
                ("aof", v) => config.aof = v.into(),
                ("port", v) => config.port = v.parse()?,
                ("threads", v) => config.threads = v.parse()?,
                _ => return Err(new_error("Field does not exist for config")),
            }
        }

        Ok(config)
    }
    pub fn aof(&self) -> &str {
        &self.aof
    }

    pub fn port(&self) -> usize {
        self.port
    }

    pub fn threads(&self) -> usize {
        self.threads
    }
}
