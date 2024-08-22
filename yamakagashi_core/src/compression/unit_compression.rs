/// unit_compress use solve algorithm for Hankel-system
/// 
/// na = b -> a is nothing (when n isn't sq mtx)
/// 
/// n^tna = n^tb
/// la = c  (n^tn def= l, n^tb def= c)
/// a = l^-1c
/// 
/// b' = na
/// 
/// |b - b'|^2
/// = |b|^2 + |b'|^2 - 2*b.dot(b')
/// = b.dot(b) + a^tn^tna - 2*b^tna
/// = b.dot(b) + c^t.dot(a) - 2*c^t.dot(a)
/// = b.dot(b) - c^t.dot(a)
/// 
/// R^2 = 1 - sse/ssd
/// R^2 = 1 - |b-b'|^2/|b-b_m|^2 , b_m is mean of b

use std::u8::MAX;
use crate::my_vector::{VecTool, HadamardProduct};

// unit transform and compression

pub fn unit_compression(b: std::iter::Take<std::iter::Skip<std::iter::Take<std::iter::StepBy<std::iter::Skip<std::slice::Iter<'_, u8>>>>>>, quality: i32) -> Vec<f32> {
    let n: usize = b.len();
    let x:Vec<f64> = 
            if n%2 == 0{(0..n).map(|i| ((-(n as i32)+1 + 2*i as i32) as f64 / 2f64)).collect::<Vec<f64>>()}
            else{(0..n).map(|i| ((-(n as i32)+1)/2 + i as i32) as f64).collect::<Vec<f64>>()}; // x == [(-n+1)/2, (-n+3)/2..(n-3)/2,(n-1)/2]
    let b_sq_norm = b.sq_norm();

    let mut a = vec![0f64; n];
    let mut c = vec![0f64; n];
    let mut l = vec![0f64; n];
    let mut f = vec![0f64; n];
    let mut power_x = vec![1f64; n];

    let mut error_1 = 0f64;
    let mut error_2 = 0f64;
    let mut error_a = 0f64;

    let mut ssd = 0f64;
    let mut sse = 0f64;

    for i in 0..n {
        let m = i / 2;
        c[i] = b.dot(&power_x);
        let temp_l = power_x.sq_norm();
        l[i] = match temp_l {
            f64::INFINITY | f64::NEG_INFINITY => {
                println!("value is INFINITY!, degree is {i}, unit size is {n}\nsqR is {}\n",1.0-sse/ssd);
                return round_to_f32(a);
            },

            _ => temp_l,
        };
        power_x.hadamard_product(&x);

        // make new f
        if i == 0 {f[0] = 1f64 / l[0];}
        else if i == 1 {f[1] = 1f64 / l[1];}
        else {
            if i % 2 == 0 {
                error_1 = l[m..2 * m].dot(f.iter().skip(0).step_by(2).take(m));
                error_2 = l[m + 1..2 * m + 1].dot(f.iter().skip(1).step_by(2).take(m));
                assert_ne!(error_1-error_2, 0.0);
                f.iter_mut().skip(0).step_by(2).take(m).for_each(|a| *a /= error_1-error_2);
                for j in 0..m {
                    let _temp = f[1 + 2 * j];
                    f[2 + 2 * j] -= _temp / (error_1-error_2);
                }
            } else {
                error_1 = l[m + 1..2 * m + 1].dot(f.iter().skip(0).step_by(2).take(m + 1));
                // error_2 = &l[m   : 2*m].dot(&f.iter().skip(0).step_by(2).take(m));
                assert_ne!(error_1-error_2, 0.0);
                f.iter_mut().skip(1).step_by(2).take(m).for_each(|a| *a /= error_2-error_1);
                for j in 0..m + 1 {
                    let _temp = f[2 * j];
                    f[1 + 2 * j] -= _temp / (error_2-error_1);
                }
            }
        }

        // make new a
        if i % 2 == 0 {
            error_a = l[m..2 * m].dot(a.iter().skip(0).step_by(2).take(m));
            for j in 0..m + 1 {
                a[2 * j] += (&c[i] - error_a) * f[i % 2 + 2 * j]
            }
        } else {
            error_a = l[m + 1..2 * m + 1].dot(a.iter().skip(1).step_by(2).take(m));
            for j in 0..m + 1 {
                a[1 + 2 * j] += (&c[i] - error_a) * f[1 + 2 * j]
            }
        }

        // quality check
        sse = &b_sq_norm - a.dot(&c);
        if i == 0 {
            ssd = sse;
            if ssd*(255.0*255.0) <= (n*(MAX as usize*MAX as usize)*13*13) as f64 {return round_to_f32(a);} // sqrt((ssd/MAX^2)/n) <= 13/255 ~ 0.05
        } else if is_quality_satisfy(quality, sse, ssd) {return round_to_f32(a);}
    }
    return round_to_f32(a);
}

fn is_quality_satisfy(quality: i32, sse: f64, ssd:f64) -> bool {
    quality as f64 *ssd <= 100.0*(ssd - sse) // quality <= 1 - SSE/SSD
}

fn round_to_f32(vec:Vec<f64>) -> Vec<f32>{
    // todo!("coeffsを何次の係数かとunitのsizeで、ある程度浮動小数点数の指数部を決めてそれとの差を記録する");
    // todo!("今は次数のみで決まってるのでunitのsizeは後で作れ");
    // println!("{:?}", vec);
    let size = vec.len();
    let mut out_vec: Vec<f32> = Vec::with_capacity(size); // coeff*(size/2)^i ~ 2^7 -> coeff ~ 2^-n? // coeff ~ 2^(7-i*(log2(size)-1)) // ln2 ~ 0.69314718056
    /*for (i, &ele) in vec.iter().enumerate() {
        // if ele > f32::MAX as f64 {panic!("bigger than f32 max!");}
        // if ele < f32::MIN as f64 {panic!("smaller than f32 min!");}
        // if ele != 0.0 && ele.abs() < f32::EPSILON as f64 {
        //     print!("smaller than f32 min!     ");
        //     out_vec.push(f32::EPSILON);
        //     continue;
        // }
        // let ex = ((ele.to_bits() >> 52) & 0x7FF) as i32 - 1023; // sign*1 exponent*11 mantissa*52
        // print!("{}, ", if ex == -1023 {0}else{ex + (6*i) as i32});
        
        // println!("{}",(6.0*i as f64).exp2());
        out_vec.push((ele*(6.0*i as f64).exp2()) as f32); // 6*i is magic number
    }*/
    vec.iter().enumerate().for_each(|(i, ele)| {
        let log_size = (size as f64).log2();
        let forecast_coeff = (i as f64 * (log_size - 1.0) - 7.0).trunc(); // coeff*(size/2)^i ~ 2^7 -> coeff ~ 2^-n? // coeff ~ 2^(7-i*(log2(size)-1))
        out_vec.push((ele * forecast_coeff.exp2()) as f32)
        // out_vec.push((ele*((6*i) as f64 .exp2())) as f32) // 6*i is magic number
    });
    out_vec
}

#[test]
fn unit_compression_test() {
    
    let test_case = 
    [72, 2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 3, 48, 128]
    ;
    let test_len = test_case.len();
    let test_iter: std::iter::Take<std::iter::Skip<std::iter::Take<std::iter::StepBy<std::iter::Skip<std::slice::Iter<'_, u8>>>>>> 
     = test_case.iter().skip(0).step_by(1).take(test_len).skip(0).take(test_len);

    let comp = unit_compression(test_iter, 85);
    println!("{:?}", comp);
    if comp[0].is_nan() {println!("NaN!")}
    else {
        println!("NOT NaN (T_T)");
        // println!("{:?}", comp)
    }

}