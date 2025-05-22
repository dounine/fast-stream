use crate::stream::Stream;
use crc32fast::Hasher;
use std::io;
use std::io::{Read, Seek, SeekFrom};

#[allow(dead_code)]
pub trait CRC32 {
    fn crc32_value(&self) -> io::Result<u32>;
}

impl CRC32 for Stream {
    fn crc32_value(&self) -> io::Result<u32> {
        let mut hasher = Hasher::new();
        self.data.borrow_mut().seek(SeekFrom::Start(0))?;
        loop {
            let mut bytes = vec![0_u8; 1024 * 1024];
            let size = self.data.borrow_mut().read(&mut bytes)?;
            hasher.update(&bytes[..size]);
            if size == 0 {
                break;
            }
        }
        Ok(hasher.finalize())
    }
}
