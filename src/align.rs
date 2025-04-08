use crate::stream::Stream;
use std::io;
use std::io::{Write};

#[allow(dead_code)]
pub trait Align {
    fn align(&mut self, align: u64) -> io::Result<()>;
}
impl Align for Stream {
    fn align(&mut self, align: u64) -> io::Result<()> {
        let len = *self.length.borrow();
        let remainder = len % align;
        if remainder != 0 {
            let padding = align - remainder;
            let padding_bytes = vec![0_u8; padding as usize];
            self.data.borrow_mut().write_all(&padding_bytes)?;
        }
        Ok(())
    }
}
