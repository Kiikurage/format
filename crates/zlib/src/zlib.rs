use crate::bit::read_bits;
use crate::deflate;
use std::io::{Error, ErrorKind};

pub fn inflate(compressed: &[u8]) -> Result<Vec<u8>, Error> {
    let compression_method = read_bits(compressed, 0, 4);
    if compression_method != 8 {
        return Err(Error::new(
            ErrorKind::InvalidData,
            "unsupported compression method",
        ));
    }

    let compressed_data_length = compressed.len() - 6;

    deflate::inflate(&compressed[2..2 + compressed_data_length])
}
