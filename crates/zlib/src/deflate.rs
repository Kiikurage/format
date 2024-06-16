use crate::bit::{read_bits, read_one_bit};
use crate::huffman_table::HuffmanTable;
use std::collections::HashMap;
use std::io::{Error, ErrorKind};

pub fn inflate(compressed: &[u8]) -> Result<Vec<u8>, Error> {
    let mut bit_offset = 0usize;
    let mut inflated = Vec::<u8>::new();

    while bit_offset >> 3 < compressed.len() {
        let is_final = read_one_bit(compressed, bit_offset) == 1;
        bit_offset += 1;

        let block_type = read_bits(compressed, bit_offset, 2);
        bit_offset += 2;

        match block_type {
            0b00 => {
                // uncompressed block
                bit_offset = inflate_uncompressed_block(compressed, &mut inflated, bit_offset)?;
            }
            0b01 => {
                // compressed with static huffman codes
                bit_offset = inflate_static_block(compressed, &mut inflated, bit_offset)?;
            }
            0b10 => {
                // compressed with dynamic huffman codes
                bit_offset = inflate_dynamic_block(compressed, &mut inflated, bit_offset)?;
            }
            _ => return Err(Error::new(ErrorKind::InvalidData, "Corrupted data")),
        }

        if is_final {
            break;
        }
    }

    Ok(inflated)
}

fn inflate_uncompressed_block(
    compressed: &[u8],
    inflated: &mut Vec<u8>,
    mut bit_offset: usize,
) -> Result<usize, Error> {
    bit_offset = (bit_offset + 7) & !0b111; // round up to byte boundary
    let mut byte_offset = bit_offset >> 3;

    let length = compressed[byte_offset] as usize + ((compressed[byte_offset + 1] as usize) << 8);
    byte_offset += 2;

    byte_offset += 2; // NLEN

    inflated.extend(&compressed[byte_offset..byte_offset + length]);
    byte_offset += length;

    bit_offset += byte_offset << 3;

    Ok(bit_offset)
}

fn inflate_static_block(
    compressed: &[u8],
    inflated: &mut Vec<u8>,
    mut bit_offset: usize,
) -> Result<usize, Error> {
    let mut code_lengths = HashMap::new();
    for i in 0..=287 {
        code_lengths.insert(
            i as u16,
            if i <= 143 {
                8
            } else if i <= 255 {
                9
            } else if i <= 279 {
                7
            } else {
                8
            },
        );
    }
    let huffman = HuffmanTable::from_code_lengths(&code_lengths);

    while bit_offset >> 3 < compressed.len() {
        let (literal_or_length_code, next_offset) = huffman.decode(compressed, bit_offset)?;
        bit_offset = next_offset;

        if literal_or_length_code <= 255 {
            inflated.push(literal_or_length_code as u8);
        } else if literal_or_length_code == 256 {
            break;
        } else {
            let extra_bits = LENGTH_EXTRA_BITS[literal_or_length_code as usize - 257];
            let length = read_bits(compressed, bit_offset, extra_bits as u8)
                + LENGTH_BASE[literal_or_length_code as usize - 257];
            bit_offset += extra_bits;

            let distance_code = read_bits(compressed, bit_offset, 5);
            bit_offset += 5;

            let extra_bits = DISTANCE_EXTRA_BITS[distance_code];
            let distance =
                read_bits(compressed, bit_offset, extra_bits as u8) + DISTANCE_BASE[distance_code];
            bit_offset += extra_bits;

            for offset in inflated.len() - distance..inflated.len() - distance + length {
                inflated.push(inflated[offset]);
            }
        }
    }

    Ok(bit_offset)
}

fn inflate_dynamic_block(
    compressed: &[u8],
    inflated: &mut Vec<u8>,
    mut bit_offset: usize,
) -> Result<usize, Error> {
    let literal_codes_count = read_bits(compressed, bit_offset, 5) + 257;
    bit_offset += 5;

    let distance_codes_count = read_bits(compressed, bit_offset, 5) + 1;
    bit_offset += 5;

    let code_length_codes_count = read_bits(compressed, bit_offset, 4) + 4;
    bit_offset += 4;

    let mut code_length_code_lengths = HashMap::new();
    for &code_length in &CODE_LENGTH_ORDER[..code_length_codes_count] {
        let code_length_code_length = read_bits(compressed, bit_offset, 3) as u8;
        bit_offset += 3;
        code_length_code_lengths.insert(code_length, code_length_code_length);
    }
    let code_length_huffman = HuffmanTable::from_code_lengths(&code_length_code_lengths);

    let mut code_lengths = Vec::new();
    let mut last_length = 0;
    loop {
        let (value, next_offset) = code_length_huffman.decode(compressed, bit_offset)?;
        bit_offset = next_offset;

        if value <= 15 {
            code_lengths.push(value as u8);
            last_length = value as u8;
        } else if value == 16 {
            let repeat_count = read_bits(compressed, bit_offset, 2) as u16 + 3;
            bit_offset += 2;
            for _ in 0..repeat_count {
                code_lengths.push(last_length)
            }
        } else if value == 17 {
            let repeat_count = read_bits(compressed, bit_offset, 3) as u16 + 3;
            bit_offset += 3;
            for _ in 0..repeat_count {
                code_lengths.push(0)
            }
        } else if value == 18 {
            let repeat_count = read_bits(compressed, bit_offset, 7) as u16 + 11;
            bit_offset += 7;
            for _ in 0..repeat_count {
                code_lengths.push(0)
            }
        }

        if code_lengths.len() == literal_codes_count + distance_codes_count {
            break;
        }
    }

    let mut literal_code_lengths = HashMap::new();
    for (value, &code_length) in code_lengths[..literal_codes_count].iter().enumerate() {
        literal_code_lengths.insert(value as u16, code_length);
    }
    let literal_codes_huffman = HuffmanTable::from_code_lengths(&literal_code_lengths);

    let mut distance_code_lengths = HashMap::new();
    for (value, &code_length) in code_lengths[literal_codes_count..].iter().enumerate() {
        distance_code_lengths.insert(value as u16, code_length);
    }
    let distance_codes_huffman = HuffmanTable::from_code_lengths(&distance_code_lengths);

    while (bit_offset >> 3) < compressed.len() {
        let (value, next_offset) = literal_codes_huffman.decode(compressed, bit_offset)?;
        bit_offset = next_offset;

        if value <= 255 {
            inflated.push(value as u8);
        } else if value == 256 {
            break;
        } else {
            let extra_bits = LENGTH_EXTRA_BITS[value as usize - 257];
            let length = read_bits(compressed, bit_offset, extra_bits as u8)
                + LENGTH_BASE[value as usize - 257];
            bit_offset += extra_bits;

            let (value, next_offset) = distance_codes_huffman.decode(compressed, bit_offset)?;
            bit_offset = next_offset;

            let extra_bits = DISTANCE_EXTRA_BITS[value as usize];
            let distance =
                read_bits(compressed, bit_offset, extra_bits as u8) + DISTANCE_BASE[value as usize];
            bit_offset += extra_bits;

            let start = inflated.len() - distance;
            for offset in 0..length {
                inflated.push(inflated[start + offset]);
            }
        }
    }

    Ok(bit_offset)
}

const CODE_LENGTH_ORDER: [u16; 19] = [
    16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
];
const LENGTH_EXTRA_BITS: [usize; 29] = [
    0, 0, 0, 0, 0, 0, 0, 0, 1, 1, // 257-266
    1, 1, 2, 2, 2, 2, 3, 3, 3, 3, // 267-276
    4, 4, 4, 4, 5, 5, 5, 5, 0, // 277-285
];
const LENGTH_BASE: [usize; 29] = [
    3, 4, 5, 6, 7, 8, 9, 10, 11, 13, // 257-266
    15, 17, 19, 23, 27, 31, 35, 43, 51, 59, // 267-276
    67, 83, 99, 115, 131, 163, 195, 227, 258, // 277-285
];
const DISTANCE_EXTRA_BITS: [usize; 30] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, // 0-9
    4, 4, 5, 5, 6, 6, 7, 7, 8, 8, // 10-19
    9, 9, 10, 10, 11, 11, 12, 12, 13, 13, // 20-29
];
const DISTANCE_BASE: [usize; 30] = [
    1, 2, 3, 4, 5, 7, 9, 13, 17, 25, // 0-9
    33, 49, 65, 97, 129, 193, 257, 385, 513, 769, // 10-19
    1025, 1537, 2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577, // 20-29
];
