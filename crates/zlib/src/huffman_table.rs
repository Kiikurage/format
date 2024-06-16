use std::collections::HashMap;
use std::io::{Error, ErrorKind};

use crate::bit::read_one_bit;

#[derive(Debug)]
pub struct HuffmanTable {
    map: HashMap<u8, HashMap<u16, u16>>,
    min_len: u8,
    max_len: u8,
}

impl HuffmanTable {
    pub fn decode(&self, buffer: &[u8], bit_offset: usize) -> Result<(u16, usize), Error> {
        let mut code: u16 = 0;
        for len in 1..=self.max_len {
            code = (code << 1) + read_one_bit(buffer, bit_offset + len as usize - 1) as u16;
            if len >= self.min_len {
                if let Some(&value) = self.map.get(&len).and_then(|m| m.get(&code)) {
                    return Ok((value, bit_offset + len as usize));
                }
            }
        }

        Err(Error::new(ErrorKind::InvalidData, "Corrupted data"))
    }

    pub fn new() -> HuffmanTable {
        HuffmanTable {
            map: HashMap::new(),
            min_len: u8::MAX,
            max_len: u8::MIN,
        }
    }

    pub fn add(&mut self, length: u8, code: u16, value: u16) {
        self.min_len = self.min_len.min(length);
        self.max_len = self.max_len.max(length);

        match self.map.get_mut(&length) {
            Some(len_map) => {
                len_map.insert(code, value);
            }
            None => {
                let mut len_map = HashMap::new();
                len_map.insert(code, value);
                self.map.insert(length, len_map);
            }
        };
    }

    pub fn from_code_lengths(lengths: &HashMap<u16, u8>) -> HuffmanTable {
        let mut table = HuffmanTable::new();

        let codes = lengths_to_codes(lengths);

        for (value, code) in codes {
            if let Some(&length) = lengths.get(&value) {
                table.add(length, code, value)
            }
        }

        table
    }
}

fn lengths_to_codes(lengths: &HashMap<u16, u8>) -> HashMap<u16, u16> {
    let mut count_by_length = [0u16; 16];
    for &length in lengths.values() {
        count_by_length[length as usize] += 1;
    }
    count_by_length[0] = 0;

    let mut code = 0u16;
    let mut next_code = [0u16; 16];
    for bits in 1..=15 {
        code = (code + count_by_length[bits - 1]) << 1;
        next_code[bits] = code;
    }

    let mut codes = HashMap::new();
    let mut values = lengths.keys().collect::<Vec<_>>();
    values.sort();

    for &value in values {
        let length = lengths[&value];
        if length != 0 {
            codes.insert(value, next_code[length as usize]);
            next_code[length as usize] += 1;
        }
    }

    codes
}

#[cfg(test)]
mod test {
    use crate::huffman_table::{lengths_to_codes, HuffmanTable};
    use std::collections::HashMap;

    #[test]
    fn test_code_from_length1() {
        let mut lengths = HashMap::new();
        lengths.insert(0, 2);
        lengths.insert(1, 1);
        lengths.insert(2, 3);
        lengths.insert(3, 3);

        let codes = lengths_to_codes(&lengths);
        assert_eq!(codes[&0], 0b10);
        assert_eq!(codes[&1], 0b0);
        assert_eq!(codes[&2], 0b110);
        assert_eq!(codes[&3], 0b111);
    }

    #[test]
    fn test_code_from_length2() {
        let mut lengths = HashMap::new();
        lengths.insert(0, 3);
        lengths.insert(1, 3);
        lengths.insert(2, 3);
        lengths.insert(3, 3);
        lengths.insert(4, 3);
        lengths.insert(5, 2);
        lengths.insert(6, 4);
        lengths.insert(7, 4);

        let codes = lengths_to_codes(&lengths);
        assert_eq!(codes[&0], 0b010);
        assert_eq!(codes[&1], 0b011);
        assert_eq!(codes[&2], 0b100);
        assert_eq!(codes[&3], 0b101);
        assert_eq!(codes[&4], 0b110);
        assert_eq!(codes[&5], 0b00);
        assert_eq!(codes[&6], 0b1110);
        assert_eq!(codes[&7], 0b1111);
    }

    #[test]
    fn test_huffman_table() {
        let mut lengths = HashMap::new();
        lengths.insert(0, 1);
        lengths.insert(1, 2);
        lengths.insert(2, 3);
        lengths.insert(3, 3);

        // 2=0b10  -> 1
        // 0=0b0   -> 0
        // 6=0b110 -> 2
        // 7=0b111 -> 3
        let huffman = HuffmanTable::from_code_lengths(&lengths);
        let codes = [0b11011010, 0b1];

        assert_eq!(huffman.decode(&codes, 0), (0, 1));
        assert_eq!(huffman.decode(&codes, 1), (1, 3));
        assert_eq!(huffman.decode(&codes, 3), (2, 6));
        assert_eq!(huffman.decode(&codes, 6), (3, 9));
    }
}
