use crate::endian::Endian;
use std::fs::File;
use std::io;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};

#[derive(Debug)]
pub enum Data {
    File(File),
    Mem(Cursor<Vec<u8>>),
}
impl Data {
    pub(crate) fn merge(&mut self, other: &mut Data) -> io::Result<u64> {
        Ok(match (self, other) {
            (Data::File(writer), Data::File(reader)) => std::io::copy(reader, writer)?,
            (Data::File(writer), Data::Mem(reader)) => std::io::copy(reader, writer)?,
            (Data::Mem(writer), Data::File(reader)) => std::io::copy(reader, writer)?,
            (Data::Mem(writer), Data::Mem(reader)) => std::io::copy(reader, writer)?,
        })
    }
}
impl From<File> for Data {
    fn from(value: File) -> Self {
        Data::File(value)
    }
}
impl From<Vec<u8>> for Data {
    fn from(value: Vec<u8>) -> Self {
        Data::Mem(Cursor::new(value))
    }
}
impl Seek for Data {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match self {
            Data::File(f) => f.seek(pos),
            Data::Mem(m) => m.seek(pos),
        }
    }
}
impl Read for Data {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            Data::File(f) => f.read(buf),
            Data::Mem(m) => m.read(buf),
        }
    }
}
impl Write for Data {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            Data::File(f) => f.write(buf),
            Data::Mem(m) => m.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            Data::File(f) => f.flush(),
            Data::Mem(m) => m.flush(),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Stream {
    pub data: Data,
    pub endian: Endian,
    pub length: u64,
    pub pins: Vec<u64>,
}
impl From<Vec<u8>> for Stream {
    fn from(value: Vec<u8>) -> Self {
        let length = value.len() as u64;
        Stream {
            data: value.into(),
            endian: Endian::Little,
            length,
            pins: vec![],
        }
    }
}
impl Stream {
    pub fn with_endian(&mut self, endian: Endian) -> &mut Self {
        self.endian = endian;
        self
    }
    pub fn empty() -> Stream {
        Self {
            data: vec![].into(),
            endian: Endian::Little,
            pins: vec![],
            length: 0,
        }
    }
    pub fn seek_start(&mut self) -> io::Result<()> {
        self.seek(SeekFrom::Start(0))?;
        Ok(())
    }
}
#[allow(dead_code)]
impl Stream {
    pub fn new(data: Data) -> Stream {
        let length = match &data {
            Data::File(f) => f.metadata().unwrap().len(),
            Data::Mem(m) => m.get_ref().len() as u64,
        };
        Self {
            data,
            endian: Endian::Little,
            pins: vec![],
            length,
        }
    }
}
impl Seek for Stream {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.data.seek(pos)
    }
}
impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.data.read(buf)
    }
}
impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.data.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.data.flush()
    }
}
#[cfg(test)]
mod tests {
    use crate::bytes::Bytes;
    use crate::stream::Stream;
    use std::io::{Seek, SeekFrom};

    #[test]
    fn test_merge() {
        let mut data = Stream::new(vec![1, 2, 3].into());
        data.seek(SeekFrom::End(0)).unwrap();
        let writer = Stream::new(vec![0, 0, 0].into());
        data.merge(writer).unwrap();
        assert_eq!(data.length, 6);
        assert_eq!(data.take_data().unwrap(), vec![1, 2, 3, 0, 0, 0]);
    }

    #[test]
    fn test_write_vec() {
        let mut data = Stream::new(vec![1, 2, 3].into());
        data.seek(SeekFrom::End(0)).unwrap();
        data.write_value(&vec![0, 0, 0]).unwrap();
        assert_eq!(data.take_data().unwrap(), vec![1, 2, 3, 0, 0, 0]);
        let mut data = Stream::new(vec![1, 2, 3].into());
        data.seek(SeekFrom::Start(0)).unwrap();
        data.write_value(&vec![0, 0, 0]).unwrap();
        assert_eq!(data.take_data().unwrap(), vec![0, 0, 0]);
        let mut data = Stream::new(vec![1, 2, 3].into());
        data.seek(SeekFrom::Start(1)).unwrap();
        data.write_value(&vec![0, 0, 0]).unwrap();
        assert_eq!(data.take_data().unwrap(), vec![1, 0, 0, 0]);
        let mut data = Stream::new(vec![1, 2, 3].into());
        data.seek(SeekFrom::Start(1)).unwrap();
        data.write_value(&vec![0]).unwrap();
        assert_eq!(data.take_data().unwrap(), vec![1, 0, 3]);
    }
}
