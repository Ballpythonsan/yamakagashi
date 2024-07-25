mod my_vector;
mod compression;
mod decompression;
use std::io::{Read, Write};
use libflate::deflate::{Decoder, Encoder};
use compression::image_compression;
use decompression::image_decompression;

// deflate yamakagashi-bytes

pub fn bitmap_to_yamakagashi(bitmap_vec:Vec<u8>, image_size:(u32, u32), quality:i32) -> Vec<u8> {
    
    let yamakagashi_bytes:Vec<u8> = image_compression(&bitmap_vec, 3, image_size, quality);

    let mut encoder = Encoder::new(Vec::new());
    encoder.write_all(&yamakagashi_bytes).unwrap();
    let deflated_yamakagashi = encoder.finish().into_result().unwrap();

    deflated_yamakagashi
}

pub fn yamakagashi_to_bitmap(deflated_yamakagashi: Vec<u8>, number_of_colors:u8, image_size:(u32, u32)) -> Vec<u8> {
    
    let mut decoder = Decoder::new(&deflated_yamakagashi[..]);
    let mut yamakagashi_bytes:Vec<u8> = Vec::new();
    decoder.read_to_end(&mut yamakagashi_bytes).unwrap();
    let bitmap_file = image_decompression(&yamakagashi_bytes, number_of_colors, image_size);

    bitmap_file
}