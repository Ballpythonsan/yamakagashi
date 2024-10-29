mod my_vector;
mod my_float;
mod compression;
mod decompression;
use std::io::{Read, Write};
use xz2::read::XzDecoder;
use xz2::write::XzEncoder;
use compression::image_compression;
use decompression::image_decompression;

// compress yamakagashi-bytes by xz

pub fn bitmap_to_yamakagashi(bitmap_vec:Vec<u8>, image_size:(u32, u32), quality:i32) -> Vec<u8> {
    
    let yamakagashi_bytes:Vec<u8> = image_compression(&bitmap_vec, 3, image_size, quality);
    
    let mut xz_yamakagashi = XzEncoder::new(Vec::new(), 6);
    xz_yamakagashi.write_all(&yamakagashi_bytes).expect("Failed to write data");
    xz_yamakagashi.finish().expect("Failed to finish compression")
}

// decompress yamakagashi-bytes by xz

pub fn yamakagashi_to_bitmap(xz_yamakagashi: Vec<u8>, number_of_colors:u8, image_size:(u32, u32)) -> Vec<u8> {
    
    let mut yamakagashi_bytes:Vec<u8> = Vec::new();
    XzDecoder::new(&xz_yamakagashi[..]).read_to_end(&mut yamakagashi_bytes).expect("Failed to read data");

    image_decompression(&yamakagashi_bytes, number_of_colors, image_size)
}