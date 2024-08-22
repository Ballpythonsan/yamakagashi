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
    // let mut unit_count = 0;
    let mut rows_turning_points: Vec<LinkedList<usize>> = vec![LinkedList::new(); size.1 as usize];
    for (i,turning_points) in rows_turning_points.iter_mut().enumerate() {
        *turning_points = turning_points_of(page.clone().skip(size.0 as usize * i).take(size.0 as usize));
        // unit_count += turning_points.len();
    }
    // println!("{:?}", rows_turning_points.iter().map(|a| a.len()).collect::<Vec<usize>>());
    // println!("unit: {unit_count}");

    for (i, (turning_points, compressed_row)) in rows_turning_points.iter().zip(compressed_page.iter_mut()).enumerate() {
        let mut pre_point: usize = 0;
        for &turning_point in turning_points {
            // if pre_point > turning_point {panic!("pre_point is bigger than turning_pint, pre_point:{pre_point}, turning_point:{turning_point}")}
            let unit = page.clone().skip(size.0 as usize*i+pre_point).take(turning_point-pre_point);
            // if i == 34 {println!("{:?}", unit.clone().collect::<Vec<_>>())}
            let coeffs: Vec<f32> = unit_compression(unit, quality);
            compressed_row.push_back(((turning_point-pre_point) as u8, coeffs));
            pre_point = turning_point;
        }
        
        let unit = page.clone().skip(size.0 as usize*i+pre_point).take(size.0 as usize-pre_point);
        let coeffs: Vec<f32> = unit_compression(unit, quality);
        compressed_row.push_back(((size.0 as usize-pre_point) as u8, coeffs));
        // if i == 34 {println!("end\n");}
        /*{if pre_point == 0 {
            println!("{}", size.0 as usize-pre_point);
            println!("{:?}",compressed_row.back().unwrap());
        }
        
        if compressed_row.iter().map(|a| a.0 as u32).sum::<u32>() != size.0 {
            println!("{:?}", turning_points.iter().map(|a| *a).collect::<Vec<usize>>());
            println!("turning points failed");
        }}*/
        assert_eq!(compressed_row.iter().map(|a| a.0 as u32).sum::<u32>(), size.0);
    }
    // organize conpressed_page
    
    compressed_page
}

fn turning_points_of<'a, I>(row: I) -> LinkedList<usize> where I: Iterator<Item = &'a u8> + ExactSizeIterator + Clone {
    let n: usize = row.len();
    let row: Vec<i32> = row.clone().map(|&x| x as i32).collect();
    // Gaussian and Laplacian filter
    let mut temp: Vec<i32> = vec![0; n];
    for i in 2..n - 2 {
        temp[i] = row[i - 2] + 2 * row[i - 1] + 4 * row[i] + 2 * row[i + 1] + row[i + 2];
    }
    let mut temp2: Vec<i32> = vec![0; n];
    for (i, ele) in (0..n).zip(row) {
        if i == 0 {
            temp2[i+1] +=   ele;
            temp2[i] += 2*ele;
            temp2[i]   += 4*ele;
            temp2[i+1] += 2*ele;
            temp2[i+2] +=   ele;
        }
        else if i == 1 {
            temp2[i-1] +=   ele;
            temp2[i-1] += 2*ele;
            temp2[i]   += 4*ele;
            temp2[i+1] += 2*ele;
            temp2[i+2] +=   ele;
        }
        else if i == n-2 {
            temp2[i-2] +=   ele;
            temp2[i-1] += 2*ele;
            temp2[i]   += 4*ele;
            temp2[i+1] += 2*ele;
            temp2[i+1] +=   ele;
        }
        else if i == n-1 {
            temp2[i-2] +=   ele;
            temp2[i-1] += 2*ele;
            temp2[i]   += 4*ele;
            temp2[i] += 2*ele;
            temp2[i-1] +=   ele;
        }
        else {
            temp2[i-2] +=   ele;
            temp2[i-1] += 2*ele;
            temp2[i]   += 4*ele;
            temp2[i+1] += 2*ele;
            temp2[i+2] +=   ele;
        }
    }
    // if temp != temp2 {println!("temp and temp2 arn't equal\n{:>4?}",temp);}
    // println!("gaussian:\n{:>4?}\n",temp2);
    
    // Apply Laplacian filter
    let mut laplacian = vec![0; n];
    for i in 1..n - 1 {
        laplacian[i] = temp2[i - 1] - 2 * temp2[i] + temp2[i + 1];
    }
    // println!("laplacian:\n{:>4?}\n",laplacian);
    
    let mut turning_points: LinkedList<usize> = LinkedList::new();
    // let threshold = 5;
    // Look for edge (zero-crossing in Laplacian)
    for i in 0..n {
        
        if i == u8::MAX as usize + match turning_points.back() {Some(x) => x,None => &0,} {turning_points.push_back(i);}
        if i == 0 || i == n-1 {continue;}
        if (laplacian[i - 1].signum() * laplacian[i].signum() == -1) || // (1 , -1), (-1, 1)
            (laplacian[i] == 0 && laplacian[i - 1].signum()*laplacian[i + 1].signum() == -1) // || // (-1, 0, 1), (1, 0, -1)
            // (laplacian[i] == 0 && laplacian[i - 1].signum()*laplacian[i + 1] ==  1 && (laplacian[i - 1]-laplacian[i + 1]).abs() > threshold) // (1, 0, 1), (-1, 0, -1)
            {
                turning_points.push_back(i);
        }
    }
    // assert!(turning_points.is_empty());
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

#[test]
fn points_test(){
    let row = vec![100,100,100,100,100,100,120,140,160,180,200,220,240,100,100,100,100,100];
    let points = turning_points_of(row.iter());

    let mut pre_point = 0;
    println!("cut row:");
    for i in points {
        print!("{:>4?} ", row.iter().skip(pre_point).take(i-pre_point).map(|a| *a).collect::<Vec<u8>>());
        pre_point = i;
    }
    print!("{:>4?} ", row.iter().skip(pre_point).take(row.len()-pre_point).map(|a| *a).collect::<Vec<u8>>());
}
