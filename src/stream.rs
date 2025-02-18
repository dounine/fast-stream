use crate::endian::Endian;
use crate::len::Len;
use crate::pin::Pin;
use std::io;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};

#[derive(Debug)]
#[allow(dead_code)]
pub struct Stream<T> {
    pub inner: T,
    pub endian: Endian,
    pub pins: Vec<u64>,
}
impl<T> Stream<T> {
    pub fn with_endian(&mut self, endian: Endian) -> &mut Self {
        self.endian = endian;
        self
    }
}
impl Stream<Cursor<Vec<u8>>> {
    pub fn take_data(&mut self) -> Vec<u8> {
        let data = &mut self.inner;
        std::mem::take(data.get_mut())
    }
    pub fn len(&self) -> io::Result<u64> {
        self.len()
    }
    pub fn align(&mut self, align: u64) -> io::Result<&mut Self> {
        let len = self.len()?;
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
impl<T: Default> Stream<T> {
    pub fn empty() -> Stream<T> {
        Self {
            inner: T::default(),
            endian: Endian::Little,
            pins: vec![],
        }
    }
}
#[allow(dead_code)]
impl<T> Stream<T> {
    pub fn new(inner: T) -> Stream<T> {
        Self {
            inner,
            endian: Endian::Little,
            pins: vec![],
        }
    }
}
impl<T: Seek> Seek for Stream<T> {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.inner.seek(pos)
    }
}
impl<T: Read> Read for Stream<T> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.inner.read(buf)
    }
}
impl<T: Write> Write for Stream<T> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.inner.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.inner.flush()
    }
}
