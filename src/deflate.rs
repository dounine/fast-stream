use crate::pin::Pin;
use crate::stream::Stream;
use miniz_oxide::deflate::compress_to_vec;
use miniz_oxide::inflate::decompress_to_vec;
use std::io;
use std::io::{Error, ErrorKind, Write};

pub use miniz_oxide::deflate::CompressionLevel;
#[allow(dead_code)]
pub trait Deflate {
    fn compress(&mut self, output: &mut Stream, level: CompressionLevel) -> io::Result<u64>;
    fn compress_self(&mut self, level: CompressionLevel) -> io::Result<u64>;
    fn un_compress(&mut self, output: &mut Stream) -> io::Result<u64>;
    fn un_compress_self(&mut self) -> io::Result<u64>;
}
impl Deflate for Stream {
    fn compress(&mut self, output: &mut Stream, level: CompressionLevel) -> io::Result<u64> {
        let data = self.copy_data()?;
        let level = match level {
            CompressionLevel::NoCompression => 0,
            CompressionLevel::BestSpeed => 1,
            CompressionLevel::BestCompression => 9,
            CompressionLevel::UberCompression => 10,
            CompressionLevel::DefaultLevel => 6,
            CompressionLevel::DefaultCompression => 0,
        };
        let compress_data = compress_to_vec(&data, level);
        let len = compress_data.len();
        output.write_all(&compress_data)?;
        Ok(len as u64)
    }
    fn compress_self(&mut self, level: CompressionLevel) -> io::Result<u64> {
        self.set_position(0)?;
        let data = self.copy_data()?;
        let level = match level {
            CompressionLevel::NoCompression => 0,
            CompressionLevel::BestSpeed => 1,
            CompressionLevel::BestCompression => 9,
            CompressionLevel::UberCompression => 10,
            CompressionLevel::DefaultLevel => 6,
            CompressionLevel::DefaultCompression => 0,
        };
        let compress_data = compress_to_vec(&data, level);
        let length = compress_data.len() as u64;
        self.set_position(0)?;
        self.data = compress_data.into();
        self.length = length;
        self.pins = vec![];
        Ok(length)
    }

    fn un_compress(&mut self, output: &mut Stream) -> io::Result<u64> {
        let data = self.copy_data()?;
        let len = data.len();
        let un_compress_data = decompress_to_vec(&data)
            .map_err(|_e| Error::new(ErrorKind::InvalidData, std::fmt::Error::default()))?;
        output.write_all(&un_compress_data)?;
        Ok(len as u64)
    }
    fn un_compress_self(&mut self) -> io::Result<u64> {
        let data = self.copy_data()?;
        let length = data.len() as u64;
        let un_compress_data = decompress_to_vec(&data)
            .map_err(|_e| Error::new(ErrorKind::InvalidData, std::fmt::Error::default()))?;
        self.set_position(0)?;
        self.data = un_compress_data.into();
        self.length = length;
        self.pins = vec![];
        Ok(length)
    }
}
