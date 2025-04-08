use crate::pin::Pin;
use crate::stream::Stream;
use crc32fast::Hasher;
use std::io;
use std::io::Read;

#[allow(dead_code)]
pub trait CRC32 {
    fn crc32_value(&mut self) -> io::Result<u32>;
}

impl CRC32 for Stream {
    fn crc32_value(&mut self) -> io::Result<u32> {
        let mut hasher = Hasher::new();
        self.set_position(0)?;
        loop {
            let mut bytes = vec![0_u8; 1024];
            let size = self.read(&mut bytes)?;
            hasher.update(&bytes[..size]);
            if size == 0 {
                break;
            }
        }
        Ok(hasher.finalize())
    }
}
