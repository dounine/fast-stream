use fast_stream::derive::NumToEnum;
use fast_stream::pin::Pin;
use fast_stream::stream::Stream;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::Write;

///
#[repr(u32)]
#[derive(Debug, NumToEnum)]
pub enum Cpu {
    X84 = 1,
    Arm = 2,
}
fn main() {
    let mut data = Stream::empty();
    // let f = std::fs::File::open("").unwrap();
    // Stream::new(Data::File(f));
    let data = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    // fs::read("./README.md").unwrap();
    let mut stream = Stream::empty();
    stream.init_sha();
    // stream.write_all(&data).unwrap();
    stream.write_value(data.clone()).unwrap();
    // stream.write(&data).unwrap();
    stream.flush().unwrap();
    let sha1 = stream.sha1_value();

    let mut sha1_hasher = Sha1::new();
    sha1_hasher.update(&data);
    let sha_1: Vec<u8> = sha1_hasher.finalize().to_vec();
    assert_eq!(sha1, sha_1);
    // let mut stream = Stream::new(Vec::with_capacity(1024).into());
    // let length = stream.length();
    // println!("{}", length);
    // let data = stream.take_data().unwrap();
    // println!("data {:?}", data);
    // let mut dd = Stream::new(vec![3, 3, 3].into());
    // data.append(&mut dd).unwrap();
    // data.seek_start().unwrap();
    // println!("{:?}", data.copy_data().unwrap());
    // let v: u32 = Cpu::Arm.into();
    // let cpu: Cpu = 3.into();
    // let data = Cursor::new(vec![0_u8, 1_u8, 2_u8]);
    // let mut stream = Stream::new(Data::Mem(data));
    // assert_eq!(stream.length(), 3);
    // stream.set_position(2).unwrap();
    // stream.write_value(1_u32).unwrap();
    // assert_eq!(stream.length(), 6);
    // println!("{:?}", stream.take_data());
}
