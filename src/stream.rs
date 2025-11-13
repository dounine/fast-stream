use crate::endian::Endian;
use crate::pin::Pin;
use crc32fast::Hasher;
use sha1::{Digest, Sha1};
use sha2::Sha256;
use std::cell::RefCell;
use std::io;
use std::io::{Cursor, Error, ErrorKind, Read, Seek, SeekFrom, Write};

#[derive(Debug)]
pub enum Data {
    #[cfg(feature = "file")]
    File {
        crc32: Option<Hasher>,
        sha1: Option<Sha1>,
        sha2: Option<Sha256>,
        data: std::fs::File,
    },
    Mem {
        crc32: Option<Hasher>,
        sha1: Option<Sha1>,
        sha2: Option<Sha256>,
        data: Cursor<Vec<u8>>,
    },
}
impl Data {
    pub(crate) fn init_sha(&mut self) {
        match self {
            #[cfg(feature = "file")]
            Data::File { sha1, sha2, .. } => {
                *sha1 = Some(Sha1::new());
                *sha2 = Some(Sha256::new());
            }
            Data::Mem { sha1, sha2, .. } => {
                *sha1 = Some(Sha1::new());
                *sha2 = Some(Sha256::new());
            }
        }
    }
    pub(crate) fn init_crc32(&mut self) {
        match self {
            #[cfg(feature = "file")]
            Data::File { crc32, .. } => {
                *crc32 = Some(Hasher::new());
            }
            Data::Mem { crc32, .. } => {
                *crc32 = Some(Hasher::new());
            }
        }
    }
    pub fn hash_update(&mut self, data: &[u8]) -> Result<(), Error> {
        match self {
            #[cfg(feature = "file")]
            Data::File {
                sha1, sha2, crc32, ..
            } => {
                if let Some(crc32) = crc32 {
                    crc32.update(data);
                }
                if let Some(sha1) = sha1 {
                    sha1.update(data);
                }
                if let Some(sha2) = sha2 {
                    sha2.update(data);
                }
            }
            Data::Mem {
                sha1, sha2, crc32, ..
            } => {
                if let Some(crc32) = crc32 {
                    crc32.update(data);
                }
                if let Some(sha1) = sha1 {
                    sha1.update(data);
                }
                if let Some(sha2) = sha2 {
                    sha2.update(data);
                }
            }
        }
        Ok(())
    }
    pub fn crc32_value(&mut self) -> u32 {
        match self {
            #[cfg(feature = "file")]
            Data::File { crc32, .. } => {
                if let Some(crc32) = crc32.take() {
                    return crc32.finalize();
                }
            }
            Data::Mem { crc32, .. } => {
                if let Some(crc32) = crc32.take() {
                    return crc32.finalize();
                }
            }
        }
        0
    }
    pub fn sha1_value(&mut self) -> Vec<u8> {
        match self {
            #[cfg(feature = "file")]
            Data::File { sha1, .. } => {
                if let Some(sha1) = sha1.take() {
                    return sha1.finalize().to_vec();
                }
            }
            Data::Mem { sha1, .. } => {
                if let Some(sha1) = sha1.take() {
                    return sha1.finalize().to_vec();
                }
            }
        }
        vec![]
    }
    pub fn sha2_value(&mut self) -> Vec<u8> {
        match self {
            #[cfg(feature = "file")]
            Data::File { sha2, .. } => {
                if let Some(sha2) = sha2.take() {
                    return sha2.finalize().to_vec();
                }
            }
            Data::Mem { sha2, .. } => {
                if let Some(sha2) = sha2.take() {
                    return sha2.finalize().to_vec();
                }
            }
        }
        vec![]
    }
    pub fn clear(&mut self) -> io::Result<()> {
        Ok(match self {
            #[cfg(feature = "file")]
            Data::File { data, .. } => {
                data.set_len(0)?;
                ()
            }
            Data::Mem { data, .. } => {
                data.set_position(0);
                data.get_mut().clear();
                ()
            }
        })
    }
    pub fn clone(&mut self) -> io::Result<Self> {
        Ok(match self {
            #[cfg(feature = "file")]
            Data::File { data: f, .. } => {
                let position = f.stream_position()?;
                let mut tmp_file = tempfile::tempfile()?;
                // tmp_file.set_len(f.metadata()?.len())?;
                std::io::copy(f, &mut tmp_file)?;
                tmp_file.seek(SeekFrom::Start(0))?;
                f.seek(SeekFrom::Start(position))?;
                tmp_file.into()
            }
            Data::Mem { data, .. } => {
                let mut data = data.clone();
                data.set_position(0);
                Data::Mem {
                    data,
                    crc32: None,
                    sha1: None,
                    sha2: None,
                }
            }
        })
    }
    pub fn copy_data(&mut self) -> io::Result<Vec<u8>> {
        let data = match self {
            #[cfg(feature = "file")]
            Data::File { data: f, .. } => {
                use std::io::Read;
                let mut data = vec![];
                f.read_to_end(&mut data)?;
                data
            }
            Data::Mem { data, .. } => data.get_ref().to_vec(),
        };
        Ok(data)
    }
    pub fn copy(&mut self, other: &mut Data) -> io::Result<u64> {
        std::io::copy(other, self)
    }
}
#[cfg(feature = "file")]
impl From<std::fs::File> for Data {
    fn from(value: std::fs::File) -> Self {
        Data::File {
            data: value,
            crc32: None,
            sha1: None,
            sha2: None,
        }
    }
}
impl From<Vec<u8>> for Data {
    fn from(value: Vec<u8>) -> Self {
        Data::Mem {
            data: Cursor::new(value),
            crc32: None,
            sha1: None,
            sha2: None,
        }
    }
}
impl Seek for Data {
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        match self {
            #[cfg(feature = "file")]
            Data::File { data, .. } => data.seek(pos),
            Data::Mem { data, .. } => data.seek(pos),
        }
    }
}
impl Read for Data {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            #[cfg(feature = "file")]
            Data::File { data, .. } => data.read(buf),
            Data::Mem { data, .. } => data.read(buf),
        }
    }
}
impl Write for Data {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            #[cfg(feature = "file")]
            Data::File {
                data,
                sha1,
                sha2,
                crc32,
                ..
            } => {
                if let Some(crc32) = crc32 {
                    crc32.update(buf);
                }
                if let Some(sha1) = sha1 {
                    sha1.update(buf);
                }
                if let Some(sha2) = sha2 {
                    sha2.update(buf);
                }
                data.write(buf)
            }
            Data::Mem {
                data,
                sha1,
                sha2,
                crc32,
                ..
            } => {
                if let Some(crc32) = crc32 {
                    crc32.update(buf);
                }
                if let Some(sha1) = sha1 {
                    sha1.update(buf);
                }
                if let Some(sha2) = sha2 {
                    sha2.update(buf);
                }
                data.write(buf)
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            #[cfg(feature = "file")]
            Data::File { data, .. } => data.flush(),
            Data::Mem { data, .. } => data.flush(),
        }
    }
}
#[derive(Debug)]
#[allow(dead_code)]
pub struct Stream {
    pub data: RefCell<Data>,
    pub endian: Endian,
    pub(crate) length: RefCell<u64>,
    pub(crate) pins: RefCell<Vec<u64>>,
}
impl Stream {
    pub fn sha1_value(&mut self) -> Vec<u8> {
        self.data.borrow_mut().sha1_value()
    }
    pub fn crc32_value(&mut self) -> u32 {
        self.data.borrow_mut().crc32_value()
    }
    pub fn sha2_value(&mut self) -> Vec<u8> {
        self.data.borrow_mut().sha2_value()
    }
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
        let data = self.data.borrow_mut().clone().unwrap();
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
    pub fn hash_computer(&mut self) -> io::Result<()> {
        self.pin()?;
        self.seek_start()?;
        loop {
            let mut bytes = [0; 1024 * 4];
            let size = self.read(&mut bytes)?;
            if size == 0 {
                break;
            }
            self.data.borrow_mut().hash_update(&bytes[..size])?;
        }
        self.un_pin()?;
        Ok(())
    }
    pub fn is_empty(&self) -> bool {
        *self.length.borrow() == 0
    }
    pub fn clone(&mut self) -> io::Result<Self> {
        self.seek_start()?;
        let mut s = Self::new(self.data.get_mut().clone()?);
        s.with_endian(self.endian.clone());
        self.seek_start()?;
        Ok(s)
    }
    pub fn clone_stream(&mut self) -> io::Result<Self> {
        self.seek_start()?;
        let mut s = Self::new(self.data.get_mut().clone()?);
        s.with_endian(self.endian.clone());
        self.seek_start()?;
        Ok(s)
    }
    pub fn copy_size_from(&mut self, stream: &mut Stream, size: usize) -> io::Result<()> {
        Ok(match stream.data.get_mut() {
            #[cfg(feature = "file")]
            Data::File { data: f, .. } => {
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
            Data::Mem { data, .. } => {
                let chunk_size = 4096;
                let mut buffer = vec![0u8; chunk_size]; // 4KB 缓冲区
                let mut copied = 0;

                while copied < size {
                    let read_size = (size - copied).min(chunk_size);
                    let n = data.read(&mut buffer[..read_size])?;
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
            Data::File { data: f, .. } => {
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
            Data::Mem { data, .. } => {
                let position = data.position() as usize;
                let mut s = Self::new(data.get_ref()[position..position + size].to_vec().into());
                s.with_endian(self.endian.clone());
                s
            }
        })
    }
    pub fn copy_empty(&self) -> io::Result<Self> {
        Ok(match &*self.data.borrow() {
            #[cfg(feature = "file")]
            Data::File { .. } => {
                let mut tmp_file = tempfile::tempfile()?;
                tmp_file.seek(SeekFrom::Start(0))?;
                let mut s = Self::new(tmp_file.into());
                s.with_endian(self.endian.clone());
                s
            }
            Data::Mem { .. } => {
                let mut s = Self::new(vec![].into());
                s.with_endian(self.endian.clone());
                s
            }
        })
    }
    pub fn copy_empty_with_capacity(&self, capacity: usize) -> io::Result<Self> {
        Ok(match &*self.data.borrow() {
            #[cfg(feature = "file")]
            Data::File { .. } => {
                let mut tmp_file = tempfile::tempfile()?;
                tmp_file.seek(SeekFrom::Start(0))?;
                let mut s = Self::new(tmp_file.into());
                s.with_endian(self.endian.clone());
                s
            }
            Data::Mem { .. } => {
                let mut s = Self::new(Vec::with_capacity(capacity).into());
                s.with_endian(self.endian.clone());
                s
            }
        })
    }
    pub fn copy_empty_same_capacity(&self) -> io::Result<Self> {
        Ok(match &*self.data.borrow() {
            #[cfg(feature = "file")]
            Data::File { .. } => {
                let mut tmp_file = tempfile::tempfile()?;
                tmp_file.seek(SeekFrom::Start(0))?;
                let mut s = Self::new(tmp_file.into());
                s.with_endian(self.endian.clone());
                s
            }
            Data::Mem { data, .. } => {
                let mut s = Self::new(Vec::with_capacity(data.get_ref().len()).into());
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
    pub fn init_sha(&mut self) {
        self.data.borrow_mut().init_sha();
    }
    pub fn init_crc32(&mut self) {
        self.data.borrow_mut().init_crc32();
    }
    pub fn new(data: Data) -> Stream {
        let length = match &data {
            #[cfg(feature = "file")]
            Data::File { data, .. } => data.metadata().unwrap().len(),
            Data::Mem { data, .. } => data.get_ref().len() as u64,
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
        let position = self.stream_position()?;
        let bytes = self.data.borrow_mut().write(buf)?;
        if *self.length.borrow() == position {
            *self.length.borrow_mut() += bytes as u64;
        } else {
            if position + bytes as u64 > *self.length.borrow() {
                // 如果写入的数据超出了当前流的长度，更新流的长度
                *self.length.borrow_mut() = position + bytes as u64;
            }
        }
        Ok(bytes)
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
