use crate::pin::Pin;
use crate::stream::Stream;
use miniz_oxide::deflate::stream::compress_stream_callback;
pub use miniz_oxide::deflate::CompressionLevel;
use miniz_oxide::deflate::{compress_to_vec, compress_to_vec_zlib};
use miniz_oxide::inflate::decompress_to_vec;
use miniz_oxide::inflate::stream::decompress_stream_callback;
use std::io;
use std::io::{Error, ErrorKind, Read, Seek, SeekFrom, Write};

#[allow(dead_code)]
pub trait Deflate {
    fn compress(&mut self, level: &CompressionLevel) -> io::Result<u64>;
    fn compress_callback(
        &mut self,
        level: &CompressionLevel,
        callback_fun: &mut impl FnMut(usize),
    ) -> io::Result<u64>;
    fn compress_zlib(&self, level: &CompressionLevel) -> io::Result<u64>;
    fn decompress(&self) -> io::Result<u64>;
    fn decompress_callback(&mut self, callback_fun: &mut impl FnMut(usize)) -> io::Result<u64>;
    fn is_zip(&self) -> io::Result<bool>;
}
impl Deflate for Stream {
    fn compress(&mut self, level: &CompressionLevel) -> io::Result<u64> {
        self.data.borrow_mut().seek(SeekFrom::Start(0))?;
        let data = self.take_data()?;
        let level = match level {
            CompressionLevel::NoCompression => 0,
            CompressionLevel::BestSpeed => 1,
            CompressionLevel::BestCompression => 9,
            CompressionLevel::UberCompression => 10,
            CompressionLevel::DefaultLevel => 6,
            CompressionLevel::DefaultCompression => 0,
        };
        let compress_data = if data.len() > 0 {
            compress_to_vec(&data, level)
        } else {
            vec![]
        };
        let length = compress_data.len() as u64;
        self.data.borrow_mut().write_all(&compress_data)?;
        *self.length.borrow_mut() = length;
        *self.pins.borrow_mut() = vec![];
        Ok(length)
    }

    fn compress_callback(
        &mut self,
        level: &CompressionLevel,
        callback_fun: &mut impl FnMut(usize),
    ) -> io::Result<u64> {
        self.data.borrow_mut().seek(SeekFrom::Start(0))?;
        let data = self.take_data()?;
        *self.length.borrow_mut() = 0;
        if data.len() > 0 {
            compress_stream_callback(&data, self, level, callback_fun)
                .map_err(|e| Error::new(ErrorKind::InvalidData, e.to_string()))?;
        }
        *self.length.borrow_mut() = self.data.borrow_mut().seek(SeekFrom::End(0))?;
        self.data.borrow_mut().seek(SeekFrom::Start(0))?;
        *self.pins.borrow_mut() = vec![];
        Ok(self.length())
    }

    fn compress_zlib(&self, level: &CompressionLevel) -> io::Result<u64> {
        self.data.borrow_mut().seek(SeekFrom::Start(0))?;
        let data = self.copy_data()?;
        let level = match level {
            CompressionLevel::NoCompression => 0,
            CompressionLevel::BestSpeed => 1,
            CompressionLevel::BestCompression => 9,
            CompressionLevel::UberCompression => 10,
            CompressionLevel::DefaultLevel => 6,
            CompressionLevel::DefaultCompression => 0,
        };
        let compress_data = if data.len() > 0 {
            compress_to_vec_zlib(&data, level)
        } else {
            vec![]
        };
        let length = compress_data.len() as u64;
        self.data.borrow_mut().clear()?;
        self.data.borrow_mut().write_all(&compress_data)?;
        *self.length.borrow_mut() = length;
        *self.pins.borrow_mut() = vec![];
        Ok(length)
    }

    fn decompress(&self) -> io::Result<u64> {
        let data = self.copy_data()?;
        let un_compress_data = decompress_to_vec(&data)
            .map_err(|_e| Error::new(ErrorKind::InvalidData, std::fmt::Error::default()))?;
        self.data.borrow_mut().clear()?;
        self.data.borrow_mut().write_all(&un_compress_data)?;
        let length = un_compress_data.len() as u64;
        *self.length.borrow_mut() = length;
        *self.pins.borrow_mut() = vec![];
        Ok(length)
    }
    fn decompress_callback(&mut self, callback_fun: &mut impl FnMut(usize)) -> io::Result<u64> {
        let data = self.take_data()?;
        *self.length.borrow_mut() = 0;
        decompress_stream_callback(&data, self, callback_fun)
            .map_err(|e| Error::new(ErrorKind::InvalidData, e.to_string()))?;
        *self.length.borrow_mut() = self.data.borrow_mut().seek(SeekFrom::End(0))?;
        self.data.borrow_mut().seek(SeekFrom::Start(0))?;
        self.seek_start()?;
        *self.pins.borrow_mut() = vec![];
        Ok(self.length())
    }

    fn is_zip(&self) -> io::Result<bool> {
        let mut bytes = [0u8; 4];
        self.pin()?;
        self.data.borrow_mut().read_exact(&mut bytes)?;
        self.un_pin()?;
        let zip_magic = [0x50, 0x4B, 0x03, 0x04];
        Ok(bytes == zip_magic)
    }
}
