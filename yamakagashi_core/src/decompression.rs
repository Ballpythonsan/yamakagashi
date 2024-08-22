/// decompress will output Vec<u8> bitmap
/// looks like lighter than compression process, but actually decompress process is lighter than compression that
/// 


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
            /*if i == 34 {
                println!("coeffs:\n{:?}",unit_coeffs);
                println!("unit:\n{:?}\n",temp_unit);
            }*/
        }
            assert_eq!(skip, size.0 as usize);
        }
    }

    image
}

fn unit_decompression(unit_size:usize, unit_coeffs:&Vec<f32>) -> Vec<u8> {

    assert_eq!(unit_size, unit_coeffs.len());
    let mut temp_unit: Vec<f64> = vec![0f64; unit_size];

    let mut zero_run_point = unit_size;
    for &coeff in unit_coeffs.iter().rev() {
        if coeff != 0f32 {break;}
        zero_run_point -= 1;
    }
    
    let x:Vec<f64> = 
        if unit_size%2 == 0{(0..unit_size).map(|i| (-(unit_size as i32)+1 + 2*i as i32) as f64/2f64).collect::<Vec<f64>>()}
        else{(0..unit_size).map(|i| ((-(unit_size as i32)+1)/2 + i as i32) as f64).collect::<Vec<f64>>()};

    let mut power_x = vec![1f64; unit_size];
    for (i, &coeff) in (0..zero_run_point).zip(unit_coeffs) {
        // temp_unit.iter_mut().zip(power_x.iter()).for_each(|(a,b)| *a += *b*coeff as f64 * (6.0*i as f64).exp2());
        let log_size = (unit_size as f64).log2();
        let forecast_coeff = (7.0 - i as f64 * (log_size - 1.0)).trunc();
        temp_unit.iter_mut().zip(power_x.iter()).for_each(|(a,b)| *a += *b*coeff as f64 * forecast_coeff.exp2());
        power_x.hadamard_product(&x);
    }

    temp_unit.iter().map(|a| {
        if *a < 0.0 {0u8}
        else if 255.0 < *a {255u8}
        else {a.round() as u8}
    }).collect()
}

fn organize(yamakagashi_bytes: &Vec<u8>, number_of_colors: u8, size: (u32, u32)) -> Vec<Vec<LinkedList<(u8, Vec<f32>)>>> {

    let mut yamakagashi: Vec<Vec<LinkedList<(u8, Vec<f32>)>>> = Vec::with_capacity(number_of_colors as usize);

    let mut index: usize = 0;
    for _ in 0..number_of_colors {

        let mut yamakagashi_row = Vec::with_capacity(size.1 as usize);
        for _ in 0..size.1 {

            let mut yamakagashi_units: LinkedList<(u8, Vec<f32>)> = LinkedList::new();
            
            let mut row_size = 0;
            while row_size < size.0 {
                let unit_size = yamakagashi_bytes[index];
                index += 1;
                let mut unit_coeffs = Vec::with_capacity(unit_size as usize);
                for _ in 0..unit_size {
                    unit_coeffs.push(f32::from_be_bytes(yamakagashi_bytes[index..index+4].try_into().unwrap()));
                    index += 4;
                }

                yamakagashi_units.push_back((unit_size, unit_coeffs));

                row_size += unit_size as u32;
            }
            if row_size != size.0 {
                println!("This is incorrect file, row size and sum all unit are not same!");
                // println!("{:?}", yamakagashi_units.iter().map(|a| a.0).collect::<Vec<u8>>());
            }
            yamakagashi_row.push(yamakagashi_units);
        }
        yamakagashi.push(yamakagashi_row);
    }

    assert_eq!(index, yamakagashi_bytes.len());

    yamakagashi
}