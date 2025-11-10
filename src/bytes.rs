use crate::endian::Endian;
use crate::pin::Pin;
use crate::stream::{Data, Stream};
use std::any::Any;
use std::arch::aarch64::uint8x8_t;
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom, Write};
use std::ops::RangeBounds;
use std::{io, ops};

pub trait StreamSized: Any + Sized {
    fn as_any(&self) -> &dyn Any {
        self
    }
}
impl StreamSized for bool {}
impl StreamSized for u8 {}
impl StreamSized for u16 {}
impl StreamSized for u32 {}
impl StreamSized for u64 {}
impl StreamSized for f32 {}
impl StreamSized for f64 {}
impl StreamSized for String {}
impl StreamSized for usize {}
impl StreamSized for isize {}

#[allow(dead_code)]
pub trait ValueRead: Sized {
    fn read(stream: &mut Stream) -> io::Result<Self> {
        Self::read_args::<bool>(stream, &None)
    }
    fn read_args<T: StreamSized>(stream: &mut Stream, args: &Option<T>) -> io::Result<Self>;
}
#[allow(dead_code)]
pub trait ValueWrite: Sized {
    fn write(self, endian: &Endian) -> io::Result<Stream> {
        self.write_args::<bool>(endian, &None)
    }
    fn write_args<T: StreamSized>(self, endian: &Endian, args: &Option<T>) -> io::Result<Stream>;
}
pub trait Bytes {
    fn append(&mut self, data: &mut Stream) -> io::Result<u64>;
    // fn merge(&mut self, dest: Stream) -> io::Result<u64>;
    fn read_value<Value: ValueRead>(&mut self) -> io::Result<Value>;
    fn read_value_args<Value: ValueRead, T: StreamSized>(
        &mut self,
        args: &Option<T>,
    ) -> io::Result<Value>;
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
        let len = *self.length.borrow() as usize;

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
    fn append(&mut self, reader: &mut Stream) -> io::Result<u64> {
        let position = self.stream_position()?;
        let mut writer = self.data.borrow_mut();
        let bytes = writer.copy(reader.data.get_mut())?;
        if *self.length.borrow() == position {
            *self.length.borrow_mut() += bytes;
        } else {
            if position + bytes > *self.length.borrow() {
                // 如果写入的数据超出了当前流的长度，更新流的长度
                *self.length.borrow_mut() = position + bytes;
            }
        }
        Ok(bytes)
    }
    // fn merge(&mut self, dest: Stream) -> io::Result<u64> {
    //     let data = dest.data;
    //     data.borrow_mut().seek(SeekFrom::Start(0))?;
    //     let position = self.stream_position()?;
    //     let bytes = self.data.borrow_mut().copy(&mut data.borrow_mut())?;
    //     if *self.length.borrow() == position {
    //         *self.length.borrow_mut() += bytes;
    //     } else {
    //         if position + bytes > *self.length.borrow() {
    //             // 如果写入的数据超出了当前流的长度，更新流的长度
    //             *self.length.borrow_mut() = position + bytes;
    //         }
    //     }
    //     Ok(bytes)
    // }
    fn read_value<Value: ValueRead>(&mut self) -> io::Result<Value> {
        Value::read(self)
    }
    fn read_value_args<Value: ValueRead, T: StreamSized>(
        &mut self,
        args: &Option<T>,
    ) -> io::Result<Value> {
        Value::read_args(self, args)
    }
    fn read_exact_size(&mut self, size: u64) -> io::Result<Vec<u8>> {
        let mut buf = vec![0_u8; size as usize];
        self.data.borrow_mut().read_exact(&mut buf)?;
        Ok(buf)
    }
    fn fill_size(&mut self, size: u64) -> io::Result<&mut Self> {
        let data_len = *self.length.borrow();
        if data_len < size {
            let diff = size - data_len;
            self.data
                .borrow_mut()
                .write_all(&vec![0_u8; diff as usize])?;
            *self.length.borrow_mut() += diff;
        }
        Ok(self)
    }

    fn drain<R: RangeBounds<usize>>(&mut self, range: R) -> io::Result<Vec<u8>> {
        let len = *self.length.borrow() as usize;
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

        match &mut self.data.get_mut() {
            #[cfg(feature = "file")]
            Data::File { data, .. } => {
                data.set_len(new_len as u64)?;
            }
            Data::Mem { data, .. } => {
                data.get_mut().truncate(new_len);
            }
        }
        self.un_pin()?;
        *self.length.borrow_mut() = new_len as u64;
        Ok(drained_data)
    }

    fn extend_from_slice(&mut self, data: &[u8]) -> io::Result<&mut Self> {
        // self.pin()?;
        let len = data.len();
        self.seek(SeekFrom::End(0))?;
        self.data.borrow_mut().write_all(data)?;
        // self.un_pin()?;
        *self.length.borrow_mut() += len as u64;
        Ok(self)
    }
    fn splice(&mut self, pos: u64, replace_with: Vec<u8>) -> io::Result<&mut Self> {
        self.pin()?;
        match &mut self.data.get_mut() {
            #[cfg(feature = "file")]
            Data::File { data: f, .. } => {
                f.seek(SeekFrom::Start(pos))?;
                // 读取插入点之后的数据
                let mut remaining_data = Vec::new();
                f.read_to_end(&mut remaining_data)?;
                f.seek(SeekFrom::Start(pos))?;
                f.write_all(&replace_with)?;
                f.write_all(&remaining_data)?;
                *self.length.borrow_mut() += replace_with.len() as u64;
            }
            Data::Mem { data, .. } => {
                *self.length.borrow_mut() += replace_with.len() as u64;
                data.get_mut()
                    .splice(pos as usize..pos as usize, replace_with);
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
        *self.length.borrow_mut() += len as u64;
        Ok(self)
    }
}
impl Stream {
    pub fn write_value<Value: ValueWrite>(&mut self, value: Value) -> io::Result<&mut Self> {
        let mut data = value.write_args::<bool>(&self.endian, &None)?;
        data.seek_start()?;
        self.append(&mut data)?;
        Ok(self)
    }
    pub fn write_value_args<Value: ValueWrite, T: StreamSized>(
        &mut self,
        value: Value,
        args: &Option<T>,
    ) -> io::Result<&mut Self> {
        let mut data = value.write_args::<T>(&self.endian, args)?;
        data.seek_start()?;
        self.append(&mut data)?;
        Ok(self)
    }
}
#[macro_export]
macro_rules! value_read {
    ($($typ:ty, $size:expr),*) => {
        $(
            impl ValueRead for $typ {
                fn read_args<T:StreamSized>(stream: &mut Stream,_args:&Option<T>) -> std::io::Result<Self> {
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
value_read!(u8, 1, i8, 1, u16, 2, i16, 2, u32, 4, i32, 4, u64, 8, i64, 8);

#[macro_export]
macro_rules! value_write {
    ($($typ:ty),*) => {
        $(
            impl ValueWrite for $typ {
                fn write_args<T:Sized>(self, endian: &crate::endian::Endian,_args:&Option<T>) -> std::io::Result<Stream> {
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
value_write!(u8, i8, u16, i16, u32, i32, u64, i64);

pub trait FromBytes<const N: usize> {
    fn from_be_bytes(data: [u8; N]) -> Self;
    fn from_le_bytes(data: [u8; N]) -> Self;
}
#[macro_export]
macro_rules! enum_to_bytes {
    ($typ:ty,$btyp:ty) => {
        impl fast_stream::bytes::ValueWrite for $typ {
            fn write_args<T: fast_stream::bytes::StreamSized>(
                self,
                endian: &fast_stream::endian::Endian,
                args: &Option<T>,
            ) -> std::io::Result<fast_stream::stream::Stream> {
                let value: $btyp = self.clone().into();
                value.write_args(endian, args)
            }
        }
        impl fast_stream::bytes::ValueRead for $typ {
            fn read_args<T: fast_stream::bytes::StreamSized>(
                stream: &mut fast_stream::stream::Stream,
                args: &Option<T>,
            ) -> std::io::Result<Self> {
                use fast_stream::bytes::Bytes;
                let value: $btyp = stream.read_value_args(args)?;
                Ok(value.into())
            }
        }
    };
}
impl ValueWrite for String {
    fn write_args<T: StreamSized>(self, _endian: &Endian, _args: &Option<T>) -> io::Result<Stream> {
        let mut data = self.as_bytes().to_vec();
        data.push(0_u8);
        Ok(data.into())
    }
}
impl ValueWrite for bool {
    fn write_args<T: StreamSized>(self, endian: &Endian, args: &Option<T>) -> io::Result<Stream> {
        u8::from(self).write_args(endian, args)
    }
}
impl ValueRead for bool {
    fn read_args<T: StreamSized>(stream: &mut Stream, args: &Option<T>) -> io::Result<Self> {
        let value = u8::read_args(stream, args)?;
        Ok(value == 1)
    }
}
impl ValueWrite for Vec<u8> {
    fn write_args<T: StreamSized>(self, _endian: &Endian, _args: &Option<T>) -> io::Result<Stream> {
        Ok(self.clone().into())
    }
}
impl ValueRead for [u8; 4] {
    fn read_args<T: StreamSized>(stream: &mut Stream, _args: &Option<T>) -> io::Result<Self> {
        let mut value = [0u8; 4];
        stream.read_exact(&mut value)?;
        Ok(value)
    }
}
impl ValueRead for String {
    fn read_args<T: StreamSized>(stream: &mut Stream, _args: &Option<T>) -> io::Result<Self> {
        let mut bytes = vec![];
        loop {
            let byte: u8 = stream.read_value()?;
            if byte == 0 {
                break;
            }
            bytes.push(byte);
        }
        String::from_utf8(bytes)
            .map_err(|e| Error::new(ErrorKind::InvalidData, format!("bytes to string {}", e)))
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
