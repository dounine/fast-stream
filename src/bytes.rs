use crate::endian::Endian;
use crate::pin::Pin;
use crate::stream::{Data, Stream};
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom, Write};
use std::ops::RangeBounds;
use std::{io, ops};

#[allow(dead_code)]
pub trait ValueRead: Sized {
    fn read(stream: &mut Stream) -> io::Result<Self>;
}
#[allow(dead_code)]
pub trait ValueWrite: Sized {
    fn write(&self, endian: &Endian) -> io::Result<Stream>;
}
pub trait Bytes {
    fn merge(&mut self, dest: Stream) -> io::Result<u64>;
    fn merge_with_pos(&mut self, dest: Stream, pos: u64) -> io::Result<u64>;
    fn read_value<Value: ValueRead>(&mut self) -> io::Result<Value>;
    fn read_exact_size(&mut self, size: u64) -> io::Result<Vec<u8>>;
    fn fill_size(&mut self, size: u64) -> io::Result<&mut Self>;
    fn drain<R: RangeBounds<usize>>(&mut self, range: R) -> io::Result<Vec<u8>>;
    fn extend_from_slice(&mut self, data: &[u8]) -> io::Result<&mut Self>;
    fn splice(&mut self, pos: u64, replace_with: Vec<u8>) -> io::Result<&mut Self>;
    fn insert_data(&mut self, data: &[u8]) -> io::Result<&mut Self>;
}
impl Stream {
    pub(crate) fn range_bounds<R: RangeBounds<usize>>(
        &mut self,
        range: R,
    ) -> io::Result<(usize, usize)> {
        let len = self.length as usize;

        let start = match range.start_bound() {
            ops::Bound::Included(&start) => start,
            ops::Bound::Excluded(start) => start.checked_add(1).ok_or(Error::new(
                ErrorKind::InvalidData,
                "attempted to index slice from after maximum usize",
            ))?,
            ops::Bound::Unbounded => 0,
        };

        let end = match range.end_bound() {
            ops::Bound::Included(end) => end.checked_add(1).ok_or(Error::new(
                ErrorKind::InvalidData,
                "attempted to index slice up to maximum usize",
            ))?,
            ops::Bound::Excluded(&end) => end,
            ops::Bound::Unbounded => len,
        };

        if start > end {
            return Err(Error::new(ErrorKind::InvalidData, "slice_index_order_fail"));
        }
        if end > len {
            return Err(Error::new(
                ErrorKind::InvalidData,
                "slice_end_index_len_fail",
            ));
        }
        Ok((start, end))
    }
}
#[allow(dead_code)]
impl Bytes for Stream {
    fn merge(&mut self, dest: Stream) -> io::Result<u64> {
        let mut data = dest.data;
        data.seek(SeekFrom::Start(0))?;
        let bytes = self.data.merge(&mut data)?;
        self.length += bytes;
        Ok(bytes)
    }
    fn merge_with_pos(&mut self, dest: Stream, pos: u64) -> io::Result<u64> {
        let mut data = dest.data;
        data.seek(SeekFrom::Start(pos))?;
        let bytes = self.data.merge(&mut data)?;
        self.length += bytes;
        Ok(bytes)
    }
    fn read_value<Value: ValueRead>(&mut self) -> io::Result<Value> {
        Value::read(self)
    }
    fn read_exact_size(&mut self, size: u64) -> io::Result<Vec<u8>> {
        let mut buf = vec![0_u8; size as usize];
        self.data.read_exact(&mut buf)?;
        Ok(buf)
    }
    fn fill_size(&mut self, size: u64) -> io::Result<&mut Self> {
        let data_len = self.length;
        if data_len < size {
            let diff = size - data_len;
            self.data.write_all(&vec![0_u8; diff as usize])?;
            self.length += diff;
        }
        Ok(self)
    }

    fn drain<R: RangeBounds<usize>>(&mut self, range: R) -> io::Result<Vec<u8>> {
        let len = self.length as usize;
        let (start, end) = self.range_bounds(range)?;
        let drain_len = end - start;
        self.pin()?;
        // 读取要移除的数据
        self.seek(SeekFrom::Start(start as u64))?;
        let mut drained_data = vec![0; drain_len];
        self.read_exact(&mut drained_data)?;
        // 读取剩余数据
        self.seek(SeekFrom::Start(end as u64))?;
        let mut remaining_data = Vec::new();
        self.read_to_end(&mut remaining_data)?;
        // 将剩余数据写回到流中
        self.seek(SeekFrom::Start(start as u64))?;
        self.write_all(&remaining_data)?;

        let new_len = len - drain_len;

        match &mut self.data {
            Data::File(f) => {
                f.set_len(new_len as u64)?;
            }
            Data::Mem(m) => {
                m.get_mut().truncate(new_len);
            }
        }
        self.un_pin()?;
        self.length = new_len as u64;
        Ok(drained_data)
    }

    fn extend_from_slice(&mut self, data: &[u8]) -> io::Result<&mut Self> {
        // self.pin()?;
        let len = data.len();
        self.seek(SeekFrom::End(0))?;
        self.data.write_all(data)?;
        // self.un_pin()?;
        self.length += len as u64;
        Ok(self)
    }
    fn splice(&mut self, pos: u64, replace_with: Vec<u8>) -> io::Result<&mut Self> {
        self.pin()?;
        match &mut self.data {
            Data::File(f) => {
                f.seek(SeekFrom::Start(pos))?;
                // 读取插入点之后的数据
                let mut remaining_data = Vec::new();
                f.read_to_end(&mut remaining_data)?;
                f.seek(SeekFrom::Start(pos))?;
                f.write_all(&replace_with)?;
                f.write_all(&remaining_data)?;
                self.length += replace_with.len() as u64;
            }
            Data::Mem(m) => {
                self.length += replace_with.len() as u64;
                m.get_mut().splice(pos as usize..pos as usize, replace_with);
            }
        }
        self.un_pin()?;
        Ok(self)
    }
    fn insert_data(&mut self, data: &[u8]) -> io::Result<&mut Self> {
        let len = data.len();
        self.seek(SeekFrom::Start(0))?;
        let mut remain_data = vec![];
        self.read_to_end(&mut remain_data)?;
        self.seek(SeekFrom::Start(0))?;
        self.write_all(data)?;
        self.write_all(&remain_data)?;
        self.length += len as u64;
        Ok(self)
    }
}
impl Stream {
    pub fn write_value<Value: ValueWrite>(&mut self, value: &Value) -> io::Result<&mut Self> {
        let data = value.write(&self.endian)?;
        let position = self.stream_position()?;
        let length = self.merge(data)?;
        if self.length == position {
            self.length += length;
        } else {
            if position + length > self.length {
                // 如果写入的数据超出了当前流的长度，更新流的长度
                self.length = position + length;
            }
        }
        Ok(self)
    }
}
#[macro_export]
macro_rules! value_read {
    ($($typ:ty, $size:expr),*) => {
        $(
            impl ValueRead for $typ {
                fn read(stream: &mut Stream) -> std::io::Result<Self> {
                    use crate::endian::Endian;
                    let mut buf = [0u8; $size];
                    stream.read_exact(&mut buf)?;
                    let value = match stream.endian {
                        Endian::Big => <$typ>::from_be_bytes(buf),
                        Endian::Little => <$typ>::from_le_bytes(buf),
                    };
                    Ok(value)
                }
            }
        )*
    }
}
value_read!(u8, 1, u16, 2, u32, 4, u64, 8);

#[macro_export]
macro_rules! value_write {
    ($($typ:ty),*) => {
        $(
            impl ValueWrite for $typ {
                fn write(&self, endian: &crate::endian::Endian) -> std::io::Result<Stream> {
                    use crate::endian::Endian;
                    let value = match endian {
                        Endian::Big => self.to_be_bytes().to_vec(),
                        Endian::Little => self.to_le_bytes().to_vec(),
                    };
                    Ok(value.into())
                }
            }
        )*
    }
}
value_write!(u8, u16, u32, u64);

pub trait FromBytes<const N: usize> {
    fn from_be_bytes(data: [u8; N]) -> Self;
    fn from_le_bytes(data: [u8; N]) -> Self;
}
// #[macro_export]
// macro_rules! from_bytes {
//     ($typ:ty,$btyp:ty,$size:expr) => {
//         impl crate::bytes::FromBytes<$size> for $typ {
//             fn from_be_bytes(data: [u8; $size]) -> Self {
//                 <$btyp>::from_be_bytes(data).into()
//             }
//
//             fn from_le_bytes(data: [u8; $size]) -> Self {
//                 <$btyp>::from_le_bytes(data).into()
//             }
//         }
//     };
// }
#[macro_export]
macro_rules! enum_to_bytes {
    ($typ:ty,$btyp:ty) => {
        impl fast_stream::bytes::ValueWrite for $typ {
            fn write(
                &self,
                endian: &fast_stream::endian::Endian,
            ) -> std::io::Result<fast_stream::stream::Stream> {
                let value: $btyp = self.clone().into();
                value.write(endian)
            }
        }
        impl fast_stream::bytes::ValueRead for $typ {
            fn read(stream: &mut fast_stream::stream::Stream) -> std::io::Result<Self> {
                use fast_stream::bytes::Bytes;
                let value: $btyp = stream.read_value()?;
                Ok(value.into())
            }
        }
    };
}
// enum_to_bytes!(A,u32);
// #[repr(u32)]
// #[derive(Debug,Clone)]
// pub enum A {
//     AA = 0,
// }
impl ValueWrite for String {
    fn write(&self, _endian: &Endian) -> io::Result<Stream> {
        Ok(self.as_bytes().to_vec().into())
    }
}
impl ValueWrite for Vec<u8> {
    fn write(&self, _endian: &Endian) -> io::Result<Stream> {
        Ok(self.clone().into())
    }
}
impl ValueRead for [u8; 4] {
    fn read(stream: &mut Stream) -> io::Result<Self> {
        let mut value = [0u8; 4];
        stream.read_exact(&mut value)?;
        Ok(value)
    }
}
#[cfg(test)]
mod test {
    use crate::bytes::Bytes;
    use crate::stream::Stream;
    use std::fs::OpenOptions;
    use std::io::{Read, Seek, SeekFrom};

    #[test]
    fn test_insert_data() {
        let mut f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open("./data.txt")
            .unwrap();
        f.seek(SeekFrom::Start(0)).unwrap();
        let mut data = vec![];
        f.read_to_end(&mut data).unwrap();
        // let mut stream = Stream::new(Data::File(f));
        // stream.insert_data(&vec![1,2,3]).unwrap();
    }
    #[test]
    fn test_vec_splice() {
        let mut stream = Stream::empty();
        stream.splice(0, vec![1, 2, 3]).unwrap();
        assert_eq!(stream.length, 3);
        assert_eq!(stream.take_data().unwrap(), vec![1, 2, 3]);
        // let mut data = vec![1, 2, 3];
        // data.splice(0..1, vec![0, 0, 0, 0]);
        // assert_eq!(data, vec![1, 0, 0])
    }
}
