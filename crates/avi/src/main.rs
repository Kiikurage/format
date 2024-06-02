use bmp::bmp::Bmp;
use riff::riff::Chunk;
use std::fs::File;
use std::io::{Error, Write};

fn main() -> Result<(), Error> {
    let avih = AVIHeader {
        micro_sec_per_frame: 1e6 as u32,
        max_bytes_per_sec: 1 * 1024 * 1024,
        padding_granularity: 0,
        flags: 0,
        total_frames: 10,
        initial_frames: 0,
        streams: 1,
        suggested_buffer_size: 1 * 1024 * 1024,
        width: 640,
        height: 426,
        reserved: [0, 0, 0, 0],
    };

    let strh = AVIStreamHeader {
        fcc_type: [b'v', b'i', b'd', b's'],
        fcc_handler: [0, 0, 0, 0],
        flags: 0,
        priority: 0,
        language: 0,
        initial_frames: 0,
        scale: 1,
        rate: 1,
        start: 0,
        length: 10,
        suggested_buffer_size: 1 * 1024 * 1024,
        quality: 0,
        sample_size: 0,
        frame_left: 0,
        frame_top: 0,
        frame_right: 640,
        frame_bottom: 426,
    };

    let strf = BitMapInfoHeader {
        size: 40,
        width: 640,
        height: 426,
        planes: 1,
        bit_count: 24,
        compression: 0, // BI_RGB
        size_image: 0,
        x_pels_per_meter: 0,
        y_pels_per_meter: 0,
        clr_used: 0,
        clr_important: 0,
    };

    let mut frames = Vec::new();
    let mut index = Vec::new();

    let bmp = Bmp::open("./resources/sample_640x426.bmp")?;
    let mut offset = 4u32;
    for _ in 0..10 {
        let size = bmp.data.len() as u32 + 8;
        index.push(AVIOldIndex {
            chunk_id: [b'0', b'0', b'd', b'c'],
            flags: AVIIF_KEYFRAME,
            offset,
            size,
        });
        offset += size;
        frames.push(Chunk::new("00dc", bmp.data.clone().to_vec()));
    }

    let mut buf = vec![0u8; 16 * index.len()].into_boxed_slice();
    for (i, entry) in index.into_iter().enumerate() {
        buf[i * 16..(i + 1) * 16]
            .copy_from_slice(&unsafe { std::mem::transmute::<AVIOldIndex, [u8; 16]>(entry) }[..]);
    }
    let idx0 = Chunk::new("idx1", buf.to_vec());

    let avi = Chunk::list_with_id(
        "RIFF",
        "AVI ",
        vec![
            Chunk::list(
                "hdrl",
                vec![
                    Chunk::new(
                        "avih",
                        (unsafe { std::mem::transmute::<AVIHeader, [u8; 56]>(avih) }).to_vec(),
                    ),
                    Chunk::list(
                        "strl",
                        vec![
                            Chunk::new(
                                "strh",
                                (unsafe { std::mem::transmute::<AVIStreamHeader, [u8; 56]>(strh) })
                                    .to_vec(),
                            ),
                            Chunk::new(
                                "strf",
                                (unsafe {
                                    std::mem::transmute::<BitMapInfoHeader, [u8; 40]>(strf)
                                })
                                .to_vec(),
                            ), // TODO
                        ],
                    ),
                    // TODO: stream header
                ],
            ),
            Chunk::list("movi", frames),
            idx0,
        ],
    );

    let mut out_buf = Vec::new();
    avi.write(&mut out_buf);
    File::create("./out.avi")?.write_all(&out_buf)?;

    Ok(())
}

/// ref: https://learn.microsoft.com/ja-jp/previous-versions/windows/desktop/api/Aviriff/ns-aviriff-avimainheader
#[repr(C, packed)]
struct AVIHeader {
    micro_sec_per_frame: u32,
    max_bytes_per_sec: u32,
    padding_granularity: u32,
    flags: u32,
    total_frames: u32,
    initial_frames: u32,
    streams: u32,
    suggested_buffer_size: u32,
    width: u32,
    height: u32,
    reserved: [u32; 4],
}

/// https://learn.microsoft.com/en-us/previous-versions/windows/desktop/api/Aviriff/ns-aviriff-avioldindex
#[repr(C, packed)]
struct AVIOldIndex {
    chunk_id: [u8; 4],
    flags: u32,
    offset: u32,
    size: u32,
}

const AVIIF_LIST: u32 = 0x1;
const AVIIF_KEYFRAME: u32 = 0x10;
const AVIIF_COMPRESSOR: u32 = 0x100;

/// https://learn.microsoft.com/en-us/previous-versions/ms779638(v=vs.85)
#[repr(C, packed)]
struct AVIStreamHeader {
    fcc_type: [u8; 4],
    fcc_handler: [u8; 4],
    flags: u32,
    priority: u16,
    language: u16,
    initial_frames: u32,
    scale: u32,
    rate: u32,
    start: u32,
    length: u32,
    suggested_buffer_size: u32,
    quality: u32,
    sample_size: u32,
    frame_left: i16,
    frame_top: i16,
    frame_right: i16,
    frame_bottom: i16,
}

/// https://learn.microsoft.com/ja-jp/windows/win32/api/wingdi/ns-wingdi-bitmapinfoheader
#[repr(C, packed)]
struct BitMapInfoHeader {
    size: u32,
    width: i32,
    height: i32,
    planes: u16,
    bit_count: u16,
    compression: u32,
    size_image: u32,
    x_pels_per_meter: i32,
    y_pels_per_meter: i32,
    clr_used: u32,
    clr_important: u32,
}
