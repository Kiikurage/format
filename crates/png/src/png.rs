use std::fs::File;
use std::io::Error;
use std::io::ErrorKind::InvalidData;
use std::mem::{size_of, transmute};
use std::os::unix::fs::FileExt;

#[derive(Default)]
pub struct Png {
    pub width: usize,
    pub height: usize,
    pub data: Box<[u8]>,
}

/// https://www.w3.org/TR/png/#5Chunk-layout
pub struct Chunk {
    length: u32,
    chunk_type: [u8; 4],
    data: Vec<u8>,
}

impl Chunk {
    fn read(file: &File, offset: u64) -> Result<Chunk, Error> {
        let mut buf4 = [0u8; 4];

        file.read_at(&mut buf4, offset)?;
        let length = u32::from_be_bytes(buf4);

        let mut chunk_type = [0u8; 4];
        file.read_at(&mut chunk_type, offset + 4)?;

        let mut data = vec![0u8; length as usize];
        file.read_at(&mut data, offset + 8)?;

        Ok(Chunk {
            length,
            chunk_type,
            data,
        })
    }
}

/// https://www.w3.org/TR/png/#11IHDR
#[repr(C, packed)]
pub struct IHDRChunk {
    width: u32,
    height: u32,
    bit_depth: u8,
    color_type: u8,
    compression_method: u8,
    filter_method: u8,
    interlace_method: u8,
}

impl Png {
    pub fn open(path: impl AsRef<std::path::Path>) -> Result<Png, Error> {
        Png::read(&File::open(path).unwrap())
    }

    pub fn read(file: &File) -> Result<Png, Error> {
        let mut offset = 8;

        // parse IHDR chunk
        let image_header_chunk = Chunk::read(file, offset)?;
        offset += image_header_chunk.length as u64 + 12;

        let mut buf = [0u8; size_of::<IHDRChunk>()];
        buf.copy_from_slice(&image_header_chunk.data);
        let image_header: IHDRChunk = unsafe { transmute(buf) };
        let width = image_header.width.to_be() as usize;
        let height = image_header.height.to_be() as usize;
        let bit_depth = image_header.bit_depth as usize;

        // collect all IDAT chunks
        let mut data_chunks = Vec::new();
        loop {
            let chunk = Chunk::read(file, offset)?;
            offset += chunk.length as u64 + 12;
            match chunk.chunk_type {
                [b'I', b'E', b'N', b'D'] => break,
                [b'I', b'D', b'A', b'T'] => {
                    data_chunks.push(chunk.data);
                }
                _ => (),
            };
        }

        // decode image data
        let mut zlib_compressed = Vec::new();
        for chunk in data_chunks {
            zlib_compressed.extend(chunk);
        }
        let data = decode_image_data(zlib_compressed, width, height, bit_depth)?;

        Ok(Png {
            width,
            height,
            data,
        })
    }
}

fn decode_image_data(
    zlib_compressed: Vec<u8>,
    width: usize,
    height: usize,
    bit_depth: usize,
) -> Result<Box<[u8]>, Error> {
    let inflated = zlib::zlib::inflate(&zlib_compressed[..])?;
    let byte_per_pixel = bit_depth / 8 * 3; // TODO;
    let byte_per_line = width * byte_per_pixel;

    let mut data = vec![0u8; height * byte_per_line];
    for y in 0..height {
        let filter_type = inflated[y * (1 + byte_per_line)];

        for x in 0..width {
            for i in 0..3 {
                let filt_x = inflated[y * (1 + byte_per_line) + 1 + x * byte_per_pixel + i];
                let recon_a = if x == 0 {
                    0u8
                } else {
                    data[y * byte_per_line + (x - 1) * byte_per_pixel + i]
                };
                let recon_b = if y == 0 {
                    0u8
                } else {
                    data[(y - 1) * byte_per_line + x * byte_per_pixel + i]
                };
                let recon_c = if y == 0 || x == 0 {
                    0u8
                } else {
                    data[(y - 1) * byte_per_line + (x - 1) * byte_per_pixel + i]
                };

                data[y * byte_per_line + x * byte_per_pixel + i] = match filter_type {
                    0 => filt_x,
                    1 => ((filt_x as u16 + recon_a as u16) & 0xff) as u8,
                    2 => ((filt_x as u16 + recon_b as u16) & 0xff) as u8,
                    3 => ((filt_x as u16 + (recon_a as u16 + recon_b as u16) / 2) & 0xff) as u8,
                    4 => {
                        ((filt_x as u16
                            + paeth_predictor(recon_a as u16, recon_b as u16, recon_c as u16))
                            & 0xff) as u8
                    }
                    _ => return Err(Error::new(InvalidData, "Unknown filter type")),
                };
            }
        }
    }

    Ok(data.into_boxed_slice())
}

fn paeth_predictor(a: u16, b: u16, c: u16) -> u16 {
    let p = (a + b) as i16 - c as i16;
    let pa = p.abs_diff(a as i16);
    let pb = p.abs_diff(b as i16);
    let pc = p.abs_diff(c as i16);

    if pa <= pb && pa <= pc {
        a
    } else if pb <= pc {
        b
    } else {
        c
    }
}
