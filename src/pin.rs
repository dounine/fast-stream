use crate::stream::Stream;
use std::fmt::Error;
use std::io;
use std::io::{ErrorKind, Seek, SeekFrom};

#[allow(dead_code)]
pub trait Pin {
    fn restore(&mut self) -> io::Result<&mut Self>;
    fn pin(&self) -> io::Result<u64>;
    fn un_pin(&self) -> io::Result<u64>;
    fn un_pin_size(&mut self, size: u64) -> io::Result<&mut Self>;
    fn set_position(&mut self, position: u64) -> io::Result<&mut Self>;
    fn position(&mut self) -> io::Result<u64>;
}
impl Pin for Stream {
    fn restore(&mut self) -> io::Result<&mut Self> {
        self.data.borrow_mut().seek(SeekFrom::Start(0))?;
        Ok(self)
    }
    fn pin(&self) -> io::Result<u64> {
        let current = self.data.borrow_mut().stream_position()?;
        self.pins.borrow_mut().push(current);
        Ok(current)
    }

    fn un_pin(&self) -> io::Result<u64> {
        if let Some(pos) = self.pins.borrow_mut().pop() {
            self.data.borrow_mut().seek(SeekFrom::Start(pos))?;
            return Ok(pos);
        }
        Err(io::Error::new(ErrorKind::NotFound, Error::default()))
    }
    /*
    恢复到pin+size位置
     */
    fn un_pin_size(&mut self, size: u64) -> io::Result<&mut Self> {
        let current_position = self.data.borrow_mut().stream_position()?;
        if let Some(position) = self.pins.borrow_mut().pop()
            && current_position - position != size
        {
            self.data
                .borrow_mut()
                .seek(SeekFrom::Start(position + size))?;
        }
        Ok(self)
    }

    fn set_position(&mut self, position: u64) -> io::Result<&mut Self> {
        self.seek(SeekFrom::Start(position))?;
        Ok(self)
    }
    fn position(&mut self) -> io::Result<u64> {
        self.stream_position()
    }
}
