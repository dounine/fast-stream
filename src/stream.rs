use crate::endian::Endian;
use std::cell::RefCell;
use std::io;
use std::io::{Cursor, Read, Seek, SeekFrom, Write};

#[derive(Debug)]
pub enum Data {
    #[cfg(feature = "file")]
    File(std::fs::File),
    Mem(Cursor<Vec<u8>>),
}
impl Data {
    pub fn copy_data(&mut self) -> io::Result<Vec<u8>> {
        let data = match self {
            #[cfg(feature = "file")]
            Data::File(f) => {
                use std::io::Read;
                let mut data = vec![];
                f.read_to_end(&mut data)?;
                data
            }
            Data::Mem(m) => {
                let mut data = vec![];
                m.read_to_end(&mut data)?;
                data
            }
        };
        Ok(data)
    }
    pub(crate) fn merge(&mut self, other: &mut Data) -> io::Result<u64> {
        Ok(match (self, other) {
            #[cfg(feature = "file")]
            (Data::File(writer), Data::File(reader)) => std::io::copy(reader, writer)?,
            #[cfg(feature = "file")]
            (Data::File(writer), Data::Mem(reader)) => std::io::copy(reader, writer)?,
            #[cfg(feature = "file")]
            (Data::Mem(writer), Data::File(reader)) => std::io::copy(reader, writer)?,
            (Data::Mem(writer), Data::Mem(reader)) => std::io::copy(reader, writer)?,
        })
    }
}
#[cfg(feature = "file")]
impl From<std::fs::File> for Data {
    fn from(value: std::fs::File) -> Self {
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
            #[cfg(feature = "file")]
            Data::File(f) => f.seek(pos),
            Data::Mem(m) => m.seek(pos),
        }
    }
}
impl Read for Data {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            #[cfg(feature = "file")]
            Data::File(f) => f.read(buf),
            Data::Mem(m) => m.read(buf),
        }
    }
}
impl Write for Data {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            #[cfg(feature = "file")]
            Data::File(f) => f.write(buf),
            Data::Mem(m) => m.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            #[cfg(feature = "file")]
            Data::File(f) => f.flush(),
            Data::Mem(m) => m.flush(),
        }
    }
}

#[derive(Debug)]
#[allow(dead_code)]
pub struct Stream {
    pub data: RefCell<Data>,
    pub endian: Endian,
    pub length: RefCell<u64>,
    pub pins: RefCell<Vec<u64>>,
}
impl From<Vec<u8>> for Stream {
    fn from(value: Vec<u8>) -> Self {
        let length = value.len() as u64;
        Stream {
            data: RefCell::new(value.into()),
            endian: Endian::Little,
            length: RefCell::new(length),
            pins: RefCell::new(vec![]),
        }
    }
}
impl Stream {
    pub fn length(&self) -> u64 {
        *self.length.borrow()
    }
    pub fn with_endian(&mut self, endian: Endian) -> &mut Self {
        self.endian = endian;
        self
    }
    pub fn empty() -> Stream {
        Self {
            data: RefCell::new(vec![].into()),
            endian: Endian::Little,
            pins: RefCell::new(vec![]),
            length: RefCell::new(0),
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
            #[cfg(feature = "file")]
            Data::File(f) => f.metadata().unwrap().len(),
            Data::Mem(m) => m.get_ref().len() as u64,
        };
        Self {
            data: RefCell::new(data),
            endian: Endian::Little,
            pins: RefCell::new(vec![]),
            length: RefCell::new(length),
        }
    }
}
impl Seek for Stream {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.data.borrow_mut().seek(pos)
    }
}
impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.data.borrow_mut().read(buf)
    }
}
impl Write for Stream {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.data.borrow_mut().write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.data.borrow_mut().flush()
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
