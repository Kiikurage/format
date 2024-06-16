pub fn read_one_bit(buffer: &[u8], bit_offset: usize) -> u8 {
    let byte = bit_offset >> 3;
    let bit = bit_offset & 0b111;
    (buffer[byte] >> bit) & 1
}

pub fn read_bits(buffer: &[u8], bit_offset: usize, bits: u8) -> usize {
    let mut value = 0usize;
    for i in 0..bits {
        let bit = read_one_bit(buffer, bit_offset + i as usize) as usize;
        value |= bit << i;
    }
    value
}

#[cfg(test)]
mod test {
    use crate::bit::read_bits;

    #[test]
    fn test_read_bit() {
        assert_eq!(read_bits(&[0b00110000], 0, 1), 0b0);
        assert_eq!(read_bits(&[0b00001100], 1, 2), 0b10);
        assert_eq!(read_bits(&[0b01000000, 0b00000010], 6, 6), 0b001001);
    }
}
