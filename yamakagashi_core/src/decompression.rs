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
            assert_eq!(row_size, size.0, "This is incorrect file, row size and sum of unit size are not same!");

            yamakagashi_row.push(yamakagashi_units);
        }
        yamakagashi.push(yamakagashi_row);
    }

    assert_eq!(index, yamakagashi_bytes.len(), "This is incorrect file, need data len and actually data len are not same!");

    yamakagashi
}

#[test]
fn unit_decompression_test() {
    let ans: Vec<i32> = vec!
    [30, 32, 35, 32, 33, 32, 34, 35, 31, 28, 32, 29, 28, 33, 33, 30, 33, 34, 29, 31, 34, 29, 28, 29, 30, 32, 30, 28, 30, 28, 29, 32, 28, 30, 34, 30, 25, 30, 29, 28, 33, 29, 25, 32, 31, 28, 33, 30, 29, 28, 25, 28, 28, 28, 29, 34, 27, 26, 33, 30, 27, 32, 29, 27, 29, 27, 27, 31, 27, 25, 30, 31, 31, 31, 30, 29, 27, 26, 26, 26, 32, 30, 27, 29, 25, 23, 28, 31, 27, 24, 26, 23, 26, 30, 28, 24, 28, 28, 28, 28, 28, 27, 26, 27, 26, 25, 25, 26, 25, 28, 27, 25, 26, 29, 26, 25, 30, 26, 22, 25, 27, 27, 23, 23, 26, 24, 27, 25, 23, 26, 30, 26, 23, 24, 27, 26, 23, 28, 29, 26, 28, 27, 25, 24, 24, 28, 24, 24, 28, 22, 22, 26, 30, 27, 22, 24, 28, 27, 28, 25, 23, 25, 27, 27, 24, 21, 24, 26, 23, 23, 24, 22, 22, 23, 22, 23, 21, 24, 25, 21, 20, 23, 25, 23, 24, 21, 22, 23, 22, 22, 23, 23, 23, 23, 21, 22, 22, 23, 23, 22, 22, 23, 23, 22, 22, 22, 21, 23, 23, 25, 23, 21, 24, 24, 23, 25, 23, 22, 25, 25, 23, 24, 23, 22, 21, 23, 23, 22, 22, 24, 27, 23]
    ;
    // let coeffs: Vec<u16> = vec!
    // [14524, 13678, 44641, 10372, 11397, 48238, 45463, 14504, 46374, 16910, 13583, 14484, 14148, 49898, 45018, 49421, 47384, 17238, 47107, 17275, 14010, 48368, 13913, 49174, 13430, 48754, 12382, 47729, 10638, 46007]
    // ;
    let coeffs: Vec<u16> = vec!
    [14657, 47264, 13348, 16187, 45601, 48731, 48971, 52474, 16292, 20565, 48862, 53498, 17906, 54319, 18028, 21274, 51305, 22056, 52272, 54454, 52042, 20597, 19974, 22140, 20567, 57336, 20745, 56115, 53818, 25669, 54984, 57537, 20097, 59482, 23254, 26305, 53804, 26778, 56007, 59264, 19444, 59718, 23259, 25380, 22631, 26594, 55559, 26637, 56189, 59976, 22720, 58977, 22898, 26836, 54479, 57588, 53945, 24704, 55095, 56961, 54940, 58799, 53778, 24758, 53384, 58057, 22190, 56965, 22262, 26034, 20495, 24147, 50428, 24151, 54325, 54561, 54255, 58024, 53219, 55816, 52596, 57005, 20082, 23640, 19909, 21671, 19661, 23144, 19070, 22735, 18737, 22075, 17576, 21469, 16405, 18798, 15970, 19688, 48138, 50138, 47624, 51228, 47422, 50858, 46679, 50758, 11863, 15336, 11355, 15894, 9369, 45880, 9260, 12082, 6419, 11371, 38951, 43776, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    ;
    let unit_size = coeffs.len();
    let value = unit_decompression(unit_size, &coeffs);

    let difference = value.iter().zip(ans.iter()).map(|(&_v, &_a)| (_v as i32 - _a)).collect::<Vec<_>>();
    let difference_sum: i32 = difference.iter().map(|_d| _d.abs() ).sum();
    
    println!("{:?}", difference);
    println!("{}", difference_sum);

}