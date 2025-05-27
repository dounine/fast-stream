use crate::endian::Endian;
use std::cell::RefCell;
use std::io;
use std::io::{Cursor, Error, ErrorKind, Read, Seek, SeekFrom, Write};

#[derive(Debug)]
pub enum Data {
    #[cfg(feature = "file")]
    File(std::fs::File),
    Mem(Cursor<Vec<u8>>),
}
impl Data {
    pub fn clear(&mut self) -> io::Result<()> {
        Ok(match self {
            #[cfg(feature = "file")]
            Data::File(f) => {
                f.set_len(0)?;
                ()
            }
            Data::Mem(f) => {
                f.set_position(0);
                f.get_mut().clear();
                ()
            }
        })
    }
    pub fn copy(&mut self) -> io::Result<Self> {
        Ok(match self {
            #[cfg(feature = "file")]
            Data::File(f) => {
                let position = f.stream_position()?;
                let mut tmp_file = tempfile::tempfile()?;
                // tmp_file.set_len(f.metadata()?.len())?;
                std::io::copy(f, &mut tmp_file)?;
                tmp_file.seek(SeekFrom::Start(0))?;
                f.seek(SeekFrom::Start(position))?;
                tmp_file.into()
            }
            Data::Mem(f) => {
                let mut data = f.clone();
                data.set_position(0);
                Data::Mem(data)
            }
        })
    }
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
                // let mut data = vec![];
                // m.read_to_end(&mut data)?;
                // data
                m.get_ref().to_vec()
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
    pub(crate) length: RefCell<u64>,
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
impl Clone for Stream {
    fn clone(&self) -> Self {
        let data = self.data.borrow_mut().copy().unwrap();
        let mut stream = Stream::new(data);
        stream.with_endian(self.endian.clone());
        stream
    }
}
impl Stream {
    pub fn clear(&self) -> io::Result<()> {
        self.data.borrow_mut().clear()?;
        *self.length.borrow_mut() = 0;
        self.pins.borrow_mut().clear();
        Ok(())
    }
    pub fn is_empty(&self) -> bool {
        *self.length.borrow() == 0
    }
    pub fn clone(&mut self) -> io::Result<Self> {
        self.seek_start()?;
        let mut s = Self::new(self.data.get_mut().copy()?);
        s.with_endian(self.endian.clone());
        self.seek_start()?;
        Ok(s)
    }
    pub fn clone_stream(&mut self) -> io::Result<Self> {
        self.seek_start()?;
        let mut s = Self::new(self.data.get_mut().copy()?);
        s.with_endian(self.endian.clone());
        self.seek_start()?;
        Ok(s)
    }
    pub fn copy_size_from(&mut self, stream: &mut Stream, size: usize) -> io::Result<()> {
        Ok(match stream.data.get_mut() {
            #[cfg(feature = "file")]
            Data::File(f) => {
                // let mut tmp_file = tempfile::tempfile()?;
                let chunk_size = 4096;
                let mut buffer = vec![0u8; chunk_size]; // 4KB 缓冲区
                let mut copied = 0;

                while copied < size {
                    let read_size = (size - copied).min(chunk_size);
                    let n = f.read(&mut buffer[..read_size])?;
                    if n == 0 {
                        break;
                    } // 数据源提前耗尽
                    self.write_value(buffer[..n].to_vec())?;
                    copied += n;
                }
                if copied < size {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "source stream not enought data",
                    ));
                }
                // tmp_file.seek(SeekFrom::Start(0))?;
                // let mut s = Self::new(tmp_file.into());
                // s.with_endian(self.endian.clone());
                // s
            }
            Data::Mem(m) => {
                let chunk_size = 4096;
                let mut buffer = vec![0u8; chunk_size]; // 4KB 缓冲区
                let mut copied = 0;

                while copied < size {
                    let read_size = (size - copied).min(chunk_size);
                    let n = m.read(&mut buffer[..read_size])?;
                    if n == 0 {
                        break;
                    } // 数据源提前耗尽
                    self.write_value(buffer[..n].to_vec())?;
                    copied += n;
                }
                if copied < size {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "source stream not enought data",
                    ));
                }
                // let position = m.position() as usize;
                // let mut s = Self::new(m.get_ref()[position..position + size].to_vec().into());
                // s.with_endian(self.endian.clone());
                // s
            }
        })
    }
    pub fn copy_size(&mut self, size: usize) -> io::Result<Self> {
        Ok(match self.data.get_mut() {
            #[cfg(feature = "file")]
            Data::File(f) => {
                let mut tmp_file = tempfile::tempfile()?;
                let chunk_size = 4096;
                let mut buffer = vec![0u8; chunk_size]; // 4KB 缓冲区
                let mut copied = 0;

                while copied < size {
                    let read_size = (size - copied).min(chunk_size);
                    let n = f.read(&mut buffer[..read_size])?;
                    if n == 0 {
                        break;
                    } // 数据源提前耗尽
                    tmp_file.write_all(&buffer[..n])?;
                    copied += n;
                }
                if copied < size {
                    return Err(Error::new(
                        ErrorKind::InvalidData,
                        "source stream not enought data",
                    ));
                }
                tmp_file.seek(SeekFrom::Start(0))?;
                let mut s = Self::new(tmp_file.into());
                s.with_endian(self.endian.clone());
                s
            }
            Data::Mem(m) => {
                let position = m.position() as usize;
                let mut s = Self::new(m.get_ref()[position..position + size].to_vec().into());
                s.with_endian(self.endian.clone());
                s
            }
        })
    }
    pub fn copy_empty(&self) -> io::Result<Self> {
        Ok(match &*self.data.borrow() {
            #[cfg(feature = "file")]
            Data::File(_) => {
                let mut tmp_file = tempfile::tempfile()?;
                tmp_file.seek(SeekFrom::Start(0))?;
                let mut s = Self::new(tmp_file.into());
                s.with_endian(self.endian.clone());
                s
            }
            Data::Mem(_) => {
                let mut s = Self::new(vec![].into());
                s.with_endian(self.endian.clone());
                s
            }
        })
    }
    pub fn copy_empty_with_capacity(&self, capacity: usize) -> io::Result<Self> {
        Ok(match &*self.data.borrow() {
            #[cfg(feature = "file")]
            Data::File(_) => {
                let mut tmp_file = tempfile::tempfile()?;
                tmp_file.seek(SeekFrom::Start(0))?;
                let mut s = Self::new(tmp_file.into());
                s.with_endian(self.endian.clone());
                s
            }
            Data::Mem(_) => {
                let mut s = Self::new(Vec::with_capacity(capacity).into());
                s.with_endian(self.endian.clone());
                s
            }
        })
    }
    pub fn copy_empty_same_capacity(&self) -> io::Result<Self> {
        Ok(match &*self.data.borrow() {
            #[cfg(feature = "file")]
            Data::File(_) => {
                let mut tmp_file = tempfile::tempfile()?;
                tmp_file.seek(SeekFrom::Start(0))?;
                let mut s = Self::new(tmp_file.into());
                s.with_endian(self.endian.clone());
                s
            }
            Data::Mem(f) => {
                let mut s = Self::new(Vec::with_capacity(f.get_ref().len()).into());
                s.with_endian(self.endian.clone());
                s
            }
        })
    }
    pub fn length(&self) -> u64 {
        *self.length.borrow()
    }
    pub fn with_endian(&mut self, endian: Endian) -> &mut Self {
        self.endian = endian;
        self
    }
    pub fn with_little_endian(&mut self) -> &mut Self {
        self.endian = Endian::Little;
        self
    }
    pub fn with_big_endian(&mut self) -> &mut Self {
        self.endian = Endian::Big;
        self
    }
    pub fn capacity(value: usize) -> Stream {
        Self {
            data: RefCell::new(Vec::with_capacity(value).into()),
            endian: Endian::Little,
            pins: RefCell::new(vec![]),
            length: RefCell::new(0),
        }
    }
    pub fn empty() -> Stream {
        Self {
            data: RefCell::new(vec![].into()),
            endian: Endian::Little,
            pins: RefCell::new(vec![]),
            length: RefCell::new(0),
        }
    }
    pub fn seek_start(&self) -> io::Result<()> {
        self.data.borrow_mut().seek(SeekFrom::Start(0))?;
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
