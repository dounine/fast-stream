use crate::endian::Endian;
use crate::stream::Stream;
use std::io;
use std::io::{Read, Seek, Write};

#[allow(dead_code)]
pub trait ValueRead<T: Read + Write + Seek>: Sized {
    fn read(stream: &mut Stream<T>) -> io::Result<Self>;
}
#[allow(dead_code)]
pub trait ValueWrite: Sized {
    fn write(&self, endian: &Endian) -> io::Result<Vec<u8>>;
}
#[allow(dead_code)]
impl<T: Read + Write + Seek> Stream<T> {
    pub fn read_value<Value: ValueRead<T>>(&mut self) -> io::Result<Value> {
        Value::read(self)
    }
    pub fn read_size(&mut self, size: u64) -> io::Result<Vec<u8>> {
        let mut buf = vec![0u8; size as usize];
        self.inner.read_exact(&mut buf)?;
        Ok(buf)
    }
}
impl<T: Read + Write + Seek> Stream<T> {
    pub fn write_value<Value: ValueWrite>(&mut self, value: &Value) -> io::Result<usize> {
        let data = value.write(&self.endian)?;
        let len = data.len();
        self.inner.write(&data)?;
        Ok(len)
    }
}
#[macro_export]
macro_rules! value_read {
    ($($typ:ty, $size:expr),*) => {
        $(
            impl<T: std::io::Read + std::io::Write + std::io::Seek> ValueRead<T> for $typ {
                fn read(stream: &mut Stream<T>) -> std::io::Result<Self> {
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
                fn write(&self, endian: &Endian) -> std::io::Result<Vec<u8>> {
                    use crate::endian::Endian;
                    let value = match endian {
                        Endian::Big => self.to_be_bytes().to_vec(),
                        Endian::Little => self.to_le_bytes().to_vec(),
                    };
                    Ok(value)
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
            fn write(&self, endian: &Endian) -> std::io::Result<Vec<u8>> {
                let value: $btyp = self.clone() as $btyp;
                value.write(endian)
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
    fn write(&self, _endian: &Endian) -> io::Result<Vec<u8>> {
        Ok(self.as_bytes().to_vec())
    }
}
impl ValueWrite for Vec<u8> {
    fn write(&self, _endian: &Endian) -> io::Result<Vec<u8>> {
        Ok(self.clone())
    }
}
impl<T: Read + Write + Seek> ValueRead<T> for [u8; 4] {
    fn read(stream: &mut Stream<T>) -> io::Result<Self> {
        let mut value = [0u8; 4];
        stream.read_exact(&mut value)?;
        Ok(value)
    }
}
