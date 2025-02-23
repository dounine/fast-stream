use crate::length::Len;
use crate::stream::Stream;
use std::io;
use std::io::{Seek, Write};

#[allow(dead_code)]
pub trait Align {
    fn align(&mut self, align: u64) -> io::Result<()>;
}
impl<T: Write + Seek> Align for Stream<T> {
    fn align(&mut self, align: u64) -> io::Result<()> {
        let len = self.length()?;
        let remainder = len % align;
        if remainder != 0 {
            let padding = align - remainder;
            let padding_bytes = vec![0_u8; padding as usize];
            self.inner.write_all(&padding_bytes)?;
        }
        Ok(())
    }
}
