#[cfg(not(feature = "bin"))]
#[derive(Debug, Clone, Eq, PartialEq)]
#[allow(dead_code)]
pub enum Endian {
    Big,
    Little,
}
#[cfg(feature = "bin")]
#[derive(bincode::Encode, bincode::Decode, Debug, Clone, Eq, PartialEq)]
#[allow(dead_code)]
pub enum Endian {
    Big,
    Little,
}