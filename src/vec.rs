use crate::pin::Pin;
use crate::stream::{Data, Stream};
use std::io;
use std::io::{Seek, SeekFrom, Write};

impl Stream {
    pub fn copy_data(&self) -> io::Result<Vec<u8>> {
        self.data.borrow_mut().flush()?;
        self.pin()?;
        let data = self.data.borrow_mut().copy_data()?;
        self.un_pin()?;
        Ok(data)
    }
    pub fn take_data(&mut self) -> io::Result<Vec<u8>> {
        Ok(match &mut self.data.get_mut() {
            #[cfg(feature = "file")]
            Data::File { data: f, .. } => {
                use std::io::Read;
                *self.length.borrow_mut() = 0;
                let mut data = Vec::new();
                f.read_to_end(&mut data)?;
                f.set_len(0)?;
                data
            }
            Data::Mem { data, .. } => {
                *self.length.borrow_mut() = 0;
                std::mem::take(data.get_mut())
            }
        })
    }
    pub fn align(&mut self, align: u64) -> io::Result<&mut Self> {
        let remainder = *self.length.borrow() % align;
        if remainder != 0 {
            let padding = align - remainder;
            self.pin()?;
            self.seek(SeekFrom::End(0))?;
            match &mut self.data.get_mut() {
                #[cfg(feature = "file")]
                Data::File { data, .. } => {
                    use std::io::Write;
                    data.write_all(&vec![0_u8; padding as usize])?;
                }
                Data::Mem { data, .. } => {
                    data.get_mut()
                        .resize((*self.length.borrow() + padding) as usize, 0u8);
                }
            }
            self.un_pin()?;
            *self.length.borrow_mut() += padding;
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
