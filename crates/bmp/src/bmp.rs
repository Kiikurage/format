use std::fs::File;
use std::io::{Error, ErrorKind};
use std::mem::size_of;
use std::os::unix::fs::FileExt;
use std::path::Path;

pub struct Bmp {
    pub data: Box<[u8]>,
    pub width: usize,
    pub height: usize,
}

#[repr(C, packed)]
pub struct BitmapFileStructure {
    pub file_type: [u8; 2],
    pub size: u32,
    pub reserved1: u16,
    pub reserved2: u16,
    pub byte_offset: u32,
}

#[repr(C, packed)]
pub struct BitmapV5Header {
    pub size: u32,
    pub width: u32,
    pub height: u32,
    pub planes: u16,
    pub bit_count: u16,
    pub compression: u32,
    pub size_image: u32,
    pub xpels_per_meter: u32,
    pub ypels_per_meter: u32,
    pub clr_used: u32,
    pub clr_important: u32,
    pub red_mask: u32,
    pub green_mask: u32,
    pub blue_mask: u32,
    pub alpha_mask: u32,
    pub cs_type: u32,
    pub endpoints: CieXYZTriple,
    pub gamma_red: u32,
    pub gamma_green: u32,
    pub gamma_blue: u32,
    pub intent: u32,
    pub profile_data: u32,
    pub profile_size: u32,
    pub reserved: u32,
}

/// https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-ciexyztriple
pub struct CieXYZTriple {
    pub cie_xyz_red: CieXYZ,
    pub cie_xyz_green: CieXYZ,
    pub cie_xyz_blue: CieXYZ,
}

/// https://learn.microsoft.com/en-us/windows/win32/api/wingdi/ns-wingdi-ciexyztriple
pub struct CieXYZ {
    pub cie_xyz_x: FxPt2Dot30,
    pub cie_xyz_y: FxPt2Dot30,
    pub cie_xyz_z: FxPt2Dot30,
}

/// Fixed-point values with a 2-bit integer part and a 30-bit fractional part
pub type FxPt2Dot30 = u32;

impl Bmp {
    pub fn open(path: impl AsRef<Path>) -> Result<Bmp, Error> {
        Bmp::read(&File::open(path).unwrap())
    }

    pub fn read(file: &File) -> Result<Bmp, Error> {
        let file_header = read_file_header(file)?;
        if file_header.file_type != [b'B', b'M'] {
            return Err(Error::new(ErrorKind::InvalidData, "Not a BMP file"));
        }

        let image_header = read_image_header(file)?;
        let width = image_header.width as usize;
        let height = image_header.height as usize;
        let byte_per_pixel = (image_header.bit_count as usize) / 8;

        let mut data = vec![0u8; width * height * byte_per_pixel].into_boxed_slice();
        file.read_at(&mut data, file_header.byte_offset as u64)?;

        Ok(Bmp {
            data,
            width,
            height,
        })
    }

    /// Normalize the image data.
    /// - Y-order from bottom-to-top to top-to-bottom
    /// - pixel format from BGR to RGB
    pub fn as_normalized_rgb(&self) -> Box<[u8]> {
        let mut rgb_data = Vec::new();
        for y in 0..self.height {
            for x in 0..self.width {
                let i = (self.height - 1 - y) * self.width + x;
                let b = self.data[i * 3];
                let g = self.data[i * 3 + 1];
                let r = self.data[i * 3 + 2];
                rgb_data.push(r);
                rgb_data.push(g);
                rgb_data.push(b);
            }
        }
        rgb_data.into_boxed_slice()
    }
}

fn read_file_header(file: &File) -> Result<BitmapFileStructure, Error> {
    let mut buf = [0u8; size_of::<BitmapFileStructure>()];
    file.read_at(&mut buf, 0)?;
    Ok(unsafe { std::mem::transmute(buf) })
}

fn read_image_header(file: &File) -> Result<BitmapV5Header, Error> {
    let mut buf = [0u8; size_of::<BitmapV5Header>()];
    file.read_at(&mut buf, size_of::<BitmapFileStructure>() as u64)?;
    Ok(unsafe { std::mem::transmute(buf) })
}
