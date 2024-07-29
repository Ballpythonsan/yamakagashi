mod unit_compression;
use unit_compression::unit_compression;
use std::collections::LinkedList;

// bitmap part of unit

pub fn image_compression(image: &[u8], number_of_colors:u8, size:(u32, u32), quality: i32) -> Vec<u8> {

    let mut yamakagashi: Vec<Vec<LinkedList<(u8, Vec<f32>)>>> = vec![Vec::new();number_of_colors as usize];

    for (which_color, compressed_page ) in (0..number_of_colors).zip(yamakagashi.iter_mut()) {
        let page = image.iter().skip(which_color as usize).step_by(number_of_colors as usize).take((size.0*size.1) as usize);
            *compressed_page = page_compression(page, size, quality);
    }

    organize(&yamakagashi, ((number_of_colors as u32)*size.0*size.1) as usize)
}

fn page_compression(page: std::iter::Take<std::iter::StepBy<std::iter::Skip<std::slice::Iter<u8>>>>, size:(u32, u32), quality: i32) -> Vec<LinkedList<(u8, Vec<f32>)>> {

    let mut compressed_page: Vec<LinkedList<(u8, Vec<f32>)>> = vec![LinkedList::new(); size.1 as usize];
    
    // let page_clone = page.clone();
    let mut rows_turning_points: Vec<LinkedList<usize>> = vec![LinkedList::new(); size.1 as usize];
    for (i,turning_points) in rows_turning_points.iter_mut().enumerate() {
        *turning_points = turning_points_of(page.clone().skip(size.0 as usize * i).take(size.0 as usize));
    }
    
    for (i, (turning_points, compressed_row)) in rows_turning_points.iter().zip(compressed_page.iter_mut()).enumerate() {
        let mut pre_point: usize = 0;
        for &turning_point in turning_points {
            if pre_point >= turning_point {panic!("pre_point is bigger than turning_pint, pre_point:{pre_point}, turning_point:{turning_point}")}
            let unit = page.clone().skip(size.0 as usize*i+pre_point).take(turning_point-pre_point);
            let coeffs: Vec<f32> = unit_compression(unit, quality);
            compressed_row.push_back(((turning_point-pre_point) as u8, coeffs));
            pre_point = turning_point;
        }

        let unit = page.clone().skip(size.0 as usize*i+pre_point).take(size.0 as usize-pre_point);
        let coeffs: Vec<f32> = unit_compression(unit, quality);
        compressed_row.push_back(((size.0 as usize-pre_point) as u8, coeffs));
    }
    // organize conpressed_page
    
    compressed_page
}

fn turning_points_of<'a, I>(row: I) -> LinkedList<usize> where I: Iterator<Item = &'a u8> + ExactSizeIterator {
    let n: usize = row.len();
    // Gaussian and Laplacian filter
    let mut temp: Vec<i32> = vec![0; n];
    for (i, &pixel) in (0..n).zip(row) {
        if i >= 2 {
            temp[i - 2] += pixel as i32;
        }
        if i < n - 1 {
            temp[i + 1] -= 2 * pixel as i32;
        }
        if i < n - 4 {
            temp[i + 4] += pixel as i32;
        }
    }

    let mut turning_points: LinkedList<usize> = LinkedList::new();
    // look for edge
    for i in 1..n {
        if (temp[i - 1].signum() == 1) ^ (temp[i].signum() == 1) {
            turning_points.push_back(i);
        }
    }

    turning_points
}

fn organize(yamakagashi: &Vec<Vec<LinkedList<(u8, Vec<f32>)>>>, pixels: usize) -> Vec<u8> {

    let pixel_bytes_size: usize = yamakagashi.len();
    const COEFF_BYTES_SIZE: usize = 4;


    let count = pixel_bytes_size*COEFF_BYTES_SIZE*pixels 
        + yamakagashi.iter().map(|page| page.iter().map(|row| row.len()).sum::<usize>()).sum::<usize>();
    let mut yamakagashi_bytes: Vec<u8> = Vec::with_capacity(count);

    for color_page in yamakagashi {
        for row in color_page {
            for (unit_size, coeffs) in row {
                yamakagashi_bytes.push(*unit_size);
                coeffs.iter().for_each(|coeff| yamakagashi_bytes.extend(coeff.to_be_bytes()));
            }
        }
    }

    yamakagashi_bytes
}
