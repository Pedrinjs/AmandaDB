use std::io::Write;

use super::value::Value;
use crate::error::Result;

pub struct Writer {
    writer: Box<dyn Write>,
}

impl Writer {
    pub fn new(writer: Box<dyn Write>) -> Self {
        Writer { writer }
    }

    pub fn write(&mut self, value: Value) -> Result<()> {
        let bytes = value.marshal();
        self.writer.write_all(&bytes)?;
        self.writer.flush()?;
        Ok(())
    }
}
