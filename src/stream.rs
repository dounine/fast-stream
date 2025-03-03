use crate::endian::Endian;
use crate::length::Len;
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
