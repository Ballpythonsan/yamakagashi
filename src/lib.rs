use std::fs::File; 
use std::io::{self, BufWriter, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use byteorder::{BigEndian, LittleEndian, ReadBytesExt, WriteBytesExt};
use yamakagashi_core::{bitmap_to_yamakagashi, yamakagashi_to_bitmap};

// file io and format

// encording
pub fn do_encode(input_path:&PathBuf, output_path:&PathBuf, quality:i32) -> io::Result<()> {

    let image_size: (u32, u32);
    let bitmap_vec: Vec<u8>;

    (image_size, bitmap_vec) = bitmap_opener(input_path)?;
    println!("width is : {}, height is : {}", image_size.0, image_size.1);
    // convert bitmap to yamakagashi
    let yamakagashi_image_data = bitmap_to_yamakagashi(bitmap_vec, image_size, quality);
    
    // edit header
    let signature = b"YAMA";
    let version = b"01";
    let width = image_size.0;
    let height = image_size.1;
    let number_of_colors = 3u8;

    // edit chunks
    let chunk_size = yamakagashi_image_data.len() as u32;

    // edit footer
    // let crc = ;
    
    // file output
    let mut output_file = BufWriter::new(File::create(output_path)?);

    // file write
    output_file.write(signature)?;
    output_file.write(version)?;
    output_file.write_u32::<BigEndian>(width)?;
    output_file.write_u32::<BigEndian>(height)?;
    output_file.write_u8(number_of_colors)?;
    output_file.write_u32::<BigEndian>(chunk_size)?;
    output_file.write_all(&yamakagashi_image_data)?;
    // output_file.write_u32::<BigEndian>(crc)?;

    output_file.flush()?;

    Ok(())
}

// decording
pub fn do_decode(input_path:&PathBuf, output_path:&PathBuf) -> io::Result<()> {

    let image_size: (u32, u32);
    let number_of_colors: u8;
    let yamakagashi_image_data: Vec<u8>;

    (image_size, number_of_colors, yamakagashi_image_data) = yamakagashi_opener(input_path)?;

    // convert yamakagashi to bitmap
    let bitmap_vec = yamakagashi_to_bitmap(yamakagashi_image_data, number_of_colors, image_size);
    
    // edit header
    let signature = b"BM";
    let header_padding = &[0u8; 12];

    let header_size = 40u32;
    let width = image_size.0;
    let height = image_size.1;
    let planes = 0u16;
    let bit_count = 24u16;

    let compression = 0u32;
    let size_image = 0u32;
    let x_pels_per_meter = 0i32;
    let y_pels_per_meter = 0i32;
    let clr_used = 0u32;
    let clr_important = 0u32;

    // file output
    let mut output_file = BufWriter::new(File::create(output_path)?);

    output_file.write(signature)?;
    output_file.write(header_padding)?;
    output_file.write_u32::<LittleEndian>(header_size)?;
    output_file.write_u32::<LittleEndian>(width)?;
    output_file.write_u32::<LittleEndian>(height)?;
    output_file.write_u16::<LittleEndian>(planes)?;
    output_file.write_u16::<LittleEndian>(bit_count)?;

    output_file.write_u32::<LittleEndian>(compression)?;
    output_file.write_u32::<LittleEndian>(size_image)?;
    output_file.write_i32::<LittleEndian>(x_pels_per_meter)?;
    output_file.write_i32::<LittleEndian>(y_pels_per_meter)?;
    output_file.write_u32::<LittleEndian>(clr_used)?;
    output_file.write_u32::<LittleEndian>(clr_important)?;

    let row_padding = &vec![0; ((width*3 + 3)/4*4) as usize];
    for row in bitmap_vec.chunks(width as usize) {
        output_file.write_all(row)?;
        output_file.write_all(row_padding)?;
    }

    output_file.flush()?;

    Ok(())
}

fn bitmap_opener(input_path:&PathBuf) -> io::Result<((u32, u32), Vec<u8>)> {

    // file input
    let mut input_file = File::open(&input_path)?;

    // bitmap file header
    // signature check
    let mut file_header = [0; 14];
    input_file.read_exact(&mut file_header)?;
    if &file_header[..2] != b"BM" {return Err(io::Error::new(io::ErrorKind::InvalidData, "Not a BMP file"));}

    let header_size = input_file.read_u32::<LittleEndian>()?;
    let width = input_file.read_i32::<LittleEndian>()?;
    if width < 0 {return Err(io::Error::new(io::ErrorKind::InvalidData, "Only positive width size BMP files are supported"));}
    let height = input_file.read_i32::<LittleEndian>()?;
    // if height > 0 {return Err(io::Error::new(io::ErrorKind::InvalidData, "Only negative height size BMP files are supported"));}
    let _planes = input_file.read_u16::<LittleEndian>()?;
    let bit_count = input_file.read_u16::<LittleEndian>()?;

    if bit_count != 24 {return Err(io::Error::new(io::ErrorKind::InvalidData, "Only 24-bit BMP files are supported"));}

    let _compression = input_file.read_u32::<LittleEndian>()?;
    let _size_image = input_file.read_u32::<LittleEndian>()?;
    let _x_pels_per_meter = input_file.read_i32::<LittleEndian>()?;
    let _y_pels_per_meter = input_file.read_i32::<LittleEndian>()?;
    let _clr_used = input_file.read_u32::<LittleEndian>()?;
    let _clr_important = input_file.read_u32::<LittleEndian>()?;

    let row_size = ((24 * width as u32 + 31) / 32) * 4;
    let pixel_row_size = (width * 3) as usize;
    let mut pixel_data = Vec::with_capacity(pixel_row_size * height as usize);

    input_file.seek(SeekFrom::Start(file_header.len() as u64 + header_size as u64))?;
    for _ in 0..height {
        let mut row = vec![0u8; row_size as usize];
        input_file.read_exact(&mut row)?;
        pixel_data.extend_from_slice(&row[0..pixel_row_size]);
    }

    Ok(((height.abs() as u32, width.abs() as u32), pixel_data))

}

fn yamakagashi_opener(input_path:&PathBuf) -> io::Result<((u32, u32), u8, Vec<u8>)> {

    // file input
    let mut input_file = File::open(&input_path)?;

    // signature check
    let mut signature = [0; 4];
    input_file.read_exact(&mut signature)?;
    if &signature != b"YAMA" {return Err(io::Error::new(io::ErrorKind::InvalidData, "Not a YAMAKAGASHI file"));}

    let mut virsion = [0; 2];
    input_file.read_exact(&mut virsion)?;
    let width = input_file.read_u32::<BigEndian>()?;
    let height = input_file.read_u32::<BigEndian>()?;
    let number_of_colors = input_file.read_u8()?;
    let chunk_size = input_file.read_u32::<BigEndian>()?;
    let mut yamakagashi_image_data = Vec::with_capacity(chunk_size as usize); 
    input_file.read_to_end(&mut yamakagashi_image_data)?;


    Ok(((width, height),number_of_colors, yamakagashi_image_data))
}