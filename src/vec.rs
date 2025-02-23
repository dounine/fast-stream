use crate::stream::Stream;
use std::io::Cursor;
use crate::length::Len;

impl Stream<Cursor<Vec<u8>>> {
    pub fn take_data(&mut self) -> Vec<u8> {
        let data = &mut self.inner;
        std::mem::take(data.get_mut())
    }
    pub fn align(&mut self, align: u64) -> std::io::Result<&mut Self> {
        let len = self.length()?;
        let data = &mut self.inner;
        let remainder = len % align;
        if remainder != 0 {
            let padding = align - remainder;
            data.get_mut().resize((len + padding) as usize, 0u8);
        }
        Ok(self)
    }
}
impl Into<Vec<u8>> for &mut Stream<Cursor<Vec<u8>>> {
    fn into(self) -> Vec<u8> {
        self.take_data()
    }
}
impl Into<Vec<u8>> for Stream<Cursor<Vec<u8>>> {
    fn into(mut self) -> Vec<u8> {
        let data = self.inner.get_mut();
        std::mem::take(data)
    }
}
