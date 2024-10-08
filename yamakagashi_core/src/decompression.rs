/// decompress will output Vec<u8> bitmap
/// looks like lighter than compression process, but actually decompress process is lighter than compression that
/// 


use crate::my_float::MyFp48;
use super::my_vector::HadamardProduct;
use std::collections::LinkedList;

// unit decompress and detransform, rebuild bitmap

pub fn image_decompression(yamakagashi_bytes: &Vec<u8>, number_of_colors: u8, size:(u32, u32)) -> Vec<u8> {
    
    let yamakagashi = organize(yamakagashi_bytes, number_of_colors, size);

    let mut image: Vec<u8> = vec![0; (size.0*size.1*number_of_colors as u32) as usize];

    for (select_color, compressed_page) in yamakagashi.iter().enumerate() {

        // let mut color_page = image.iter().skip(color).step_by(number_of_colors).take(size.0*size.1);
        for (i, page_row) in compressed_page.iter().enumerate() {

            let mut skip = 0;
            for (unit_size, unit_coeffs) in page_row {
                let temp_unit = unit_decompression(*unit_size as usize, unit_coeffs);
                image.iter_mut().skip(select_color).step_by(number_of_colors as usize) // select color
                .skip(i*size.0 as usize) // select row
                .skip(skip).take(*unit_size as usize) // select unit
                .zip(temp_unit.iter()).for_each(|(a, b)| *a = *b);
            skip += *unit_size as usize;
        }
            assert_eq!(skip, size.0 as usize);
        }
    }

    image
}

fn unit_decompression(unit_size:usize, unit_coeffs:&Vec<u16>) -> Vec<u8> {

    assert_eq!(unit_size, unit_coeffs.len());
    let mut temp_unit: Vec<MyFp48> = vec![MyFp48::ZERO; unit_size];

    let mut zero_run_point = unit_size;
    for &coeff in unit_coeffs.iter().rev() {
        if coeff != 0u16 {break;}
        zero_run_point -= 1;
    }
    
    let x:Vec<MyFp48> = (0..unit_size).map(|i| MyFp48::new((-(unit_size as i32)+1 + 2*i as i32) as f32 / 2.0)).collect(); // x == [(-n+1)/2, (-n+3)/2..(n-3)/2,(n-1)/2]
    let mut power_x = vec![MyFp48::ONE; unit_size];
    for (i, &coeff) in (0..zero_run_point).zip(unit_coeffs) {
        let log_size = (unit_size as f64).log2();
        let forecast_coeff = (7.0 - i as f64 * (log_size - 1.0)).trunc() as i32;
        let actuall_coeff = MyFp48::from_record_bytes(coeff) * MyFp48::exp2(forecast_coeff);

        temp_unit.iter_mut().zip(power_x.iter()).for_each(|(a,b)| *a += *b*actuall_coeff);
        power_x.hadamard_product(&x);
    }

    temp_unit.iter().map(|a| {
        match a.round_u8() {
            Ok(value) => value,
            Err(_) => panic!("can't round u8, because too big"),
        }
    } ).collect()
}

fn organize(yamakagashi_bytes: &Vec<u8>, number_of_colors: u8, size: (u32, u32)) -> Vec<Vec<LinkedList<(u16, Vec<u16>)>>> {

    let mut yamakagashi: Vec<Vec<LinkedList<(u16, Vec<u16>)>>> = Vec::with_capacity(number_of_colors as usize);

    let mut index: usize = 0;
    for _ in 0..number_of_colors {

        let mut yamakagashi_row = Vec::with_capacity(size.1 as usize);
        for _ in 0..size.1 {

            let mut yamakagashi_units: LinkedList<(u16, Vec<u16>)> = LinkedList::new();
            
            let mut row_size = 0;
            while row_size < size.0 {
                let unit_size = u16::from_be_bytes(yamakagashi_bytes[index..index+2].try_into().unwrap());
                index += 2;
                let mut unit_coeffs = Vec::with_capacity(unit_size as usize);
                for _ in 0..unit_size {
                    unit_coeffs.push(u16::from_be_bytes(yamakagashi_bytes[index..index+2].try_into().unwrap()));
                    index += 2;
                }

                yamakagashi_units.push_back((unit_size, unit_coeffs));

                row_size += unit_size as u32;
            }
            if row_size != size.0 {
                panic!("This is incorrect file, row size and sum of unit size are not same!");
            }
            yamakagashi_row.push(yamakagashi_units);
        }
        yamakagashi.push(yamakagashi_row);
    }

    assert_eq!(index, yamakagashi_bytes.len());

    yamakagashi
}