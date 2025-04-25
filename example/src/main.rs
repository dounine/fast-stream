use fast_stream::pin::Pin;
use fast_stream::stream::{Data, Stream};
use std::io::{Cursor};

fn main() {
    let data = Cursor::new(vec![0_u8, 1_u8, 2_u8]);
    let mut stream = Stream::new(Data::Mem(data));
    assert_eq!(stream.length(), 3);
    stream.set_position(2).unwrap();
    stream.write_value(1_u32).unwrap();
    assert_eq!(stream.length(), 6);
    println!("{:?}", stream.take_data());
}
