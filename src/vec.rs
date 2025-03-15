use crate::pin::Pin;
use crate::stream::{Data, Stream};
use std::io;
use std::io::{Seek, SeekFrom};

impl Stream {
    pub fn take_data(&mut self) -> io::Result<Vec<u8>> {
        Ok(match &mut self.data {
            #[cfg(feature = "file")]
            Data::File(f) => {
                use std::io::{Read};
                self.length = 0;
                let mut data = Vec::new();
                f.read_to_end(&mut data)?;
                f.set_len(0)?;
                data
            }
            Data::Mem(m) => {
                self.length = 0;
                std::mem::take(m.get_mut())
            }
        })
    }
    pub fn align(&mut self, align: u64) -> io::Result<&mut Self> {
        let remainder = self.length % align;
        if remainder != 0 {
            let padding = align - remainder;
            self.pin()?;
            self.seek(SeekFrom::End(0))?;
            match &mut self.data {
                #[cfg(feature = "file")]
                Data::File(f) => {
                    use std::io::{Write};
                    f.write_all(&vec![0_u8; padding as usize])?;
                }
                Data::Mem(m) => {
                    m.get_mut().resize((self.length + padding) as usize, 0u8);
                }
            }
            self.un_pin()?;
            self.length += padding;
        }
        Ok(self)
    }
}
impl Into<io::Result<Vec<u8>>> for &mut Stream {
    fn into(self) -> io::Result<Vec<u8>> {
        self.take_data()
    }
}
impl Into<io::Result<Vec<u8>>> for Stream {
    fn into(mut self) -> io::Result<Vec<u8>> {
        self.take_data()
    }
}
