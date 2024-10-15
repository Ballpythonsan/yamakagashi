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

use crate::my_float::MyFp48;
use crate::my_vector::{VecTool, HadamardProduct};

// unit transform and compression

pub fn unit_compression(b: std::iter::Take<std::iter::Skip<std::iter::Take<std::iter::StepBy<std::iter::Skip<std::slice::Iter<'_, u8>>>>>>, quality: i32) -> Vec<u16> {
    let n: usize = b.len();
    let x:Vec<MyFp48> = (0..n).map(|i| MyFp48::new((-(n as i32)+1 + 2*i as i32) as f32 / 2.0)).collect(); // x == [(-n+1)/2, (-n+3)/2..(n-3)/2,(n-1)/2]
    let b_sq_norm = b.sq_norm();

    let mut a = vec![MyFp48::ZERO; n];
    let mut c = vec![MyFp48::ZERO; n];
    let mut l = vec![MyFp48::ZERO; n];
    let mut f = vec![MyFp48::ZERO; n];
    let mut power_x = vec![MyFp48::ONE; n];

    // let mut error_1 = MyFp48::ZERO;
    let mut error_2 = MyFp48::ZERO; // push in for-loop later
    //let mut error_a = MyFp48::ZERO;

    let mut ssd = MyFp48::ZERO;
    let mut sse = MyFp48::ZERO;

    for i in 0..n {
        let m = i / 2;
        c[i] = b.dot(power_x.iter());
        let temp_l = power_x.sq_norm();
        l[i] = if temp_l.is_normal() || temp_l.is_zero() { temp_l }
        else { panic!("value is NORMAL!, degree is {i}, unit size is {n}"); };
        power_x.hadamard_product(&x);

        // make new f
        if i == 0 {f[0] = MyFp48::ONE / l[0];}
        else if i == 1 {f[1] = MyFp48::ONE / l[1];}
        else {
            if i % 2 == 0 {
                let error_1 = l[m..2 * m].dot(f.iter().skip(0).step_by(2).take(m));
                error_2 = l[m + 1..2 * m + 1].dot(f.iter().skip(1).step_by(2).take(m));
                assert_ne!(error_1-error_2, MyFp48::ZERO);
                f.iter_mut().skip(0).step_by(2).take(m).for_each(|a| *a /= error_1-error_2);
                for j in 0..m {
                    let _temp = f[1 + 2 * j];
                    f[2 + 2 * j] -= _temp / (error_1-error_2);
                }
            } else {
                let error_1 = l[m + 1..2 * m + 1].dot(f.iter().skip(0).step_by(2).take(m + 1));
                // error_2 = &l[m   : 2*m].dot(&f.iter().skip(0).step_by(2).take(m));
                assert_ne!(error_1-error_2, MyFp48::ZERO);
                f.iter_mut().skip(1).step_by(2).take(m).for_each(|a| *a /= error_2-error_1);
                for j in 0..m + 1 {
                    let _temp = f[2 * j];
                    f[1 + 2 * j] -= _temp / (error_2-error_1);
                }
            }
        }

        // make new a
        if i % 2 == 0 {
            let error_a = l[m..2 * m].dot(a.iter().skip(0).step_by(2).take(m));
            for j in 0..m + 1 {
                a[2 * j] += (c[i] - error_a) * f[i % 2 + 2 * j]
            }
        } else {
            let error_a = l[m + 1..2 * m + 1].dot(a.iter().skip(1).step_by(2).take(m));
            for j in 0..m + 1 {
                a[1 + 2 * j] += (c[i] - error_a) * f[1 + 2 * j]
            }
        }

        // quality check
        sse = b_sq_norm - a.dot(c.iter());
        if i == 0 {
            ssd = sse;
            if ssd <= MyFp48::new((n*13*13) as f32) { // sqrt((ssd/MAX^2)/n) <= 13/255 ~ 0.05
                return round_to_record_u16(a);
            }
        } else if is_quality_satisfy(quality, sse, ssd) {
            return round_to_record_u16(a);
        }
    }

    println!("quality isn't satisfy (T_T) final quality is: {:.3}", MyFp48::ONE - sse/ssd);
    return round_to_record_u16(a);
}

fn is_quality_satisfy(quality: i32, sse: MyFp48, ssd:MyFp48) -> bool {
    MyFp48::new(quality as f32) *ssd <= MyFp48::new(100.0)*(ssd - sse) // quality/100 <= 1 - SSE/SSD
    // quality as MyFp48/100.0 <= 1.0 - sse/ssd // quality/100 <= 1 - SSE/SSD
}

fn round_to_record_u16(vec:Vec<MyFp48>) -> Vec<u16>{

    let size = vec.len();
    let mut out_vec: Vec<u16> = Vec::with_capacity(size); // coeff*(size/2)^i ~ 2^7 -> coeff ~ 2^-n? // coeff ~ 2^(7-i*(log2(size)-1))

    // when is vec constant functions-coeffs
    let is_constant: bool = vec.iter().skip(1).fold(true, |acc, ele| acc & ele.is_zero() );
    if is_constant {

        let adjusted_coeff = match vec[0].round_u8() {
            Ok(value) => {
                MyFp48::new(value as f32)*MyFp48::exp2(-7)
            },
            Err(_) => panic!("can't round u8, because too big"),
        };

        out_vec = vec![0; size];
        out_vec[0] = match adjusted_coeff.to_record_bytes() {
            Ok(record_f32) => record_f32,
            Err("can't express f32, because of this MyFp48 abs is too small") => 0x0000,
            Err("can't express f32, because of this MyFp48 abs is too big") => {
                println!("can't express f32, because of this MyFp48 is too big");

                println!("returns record f32 max instead");
                // let record_u16_max: u16 = 
                if adjusted_coeff.sign() == 1 { 0x7FFF }
                else { 0xFFFF }
            },
            _ => panic!("may can't see you"),
        };

        return out_vec;
    }
    
    vec.iter().enumerate().for_each(|(i, coeff)| {

        let log_size = (size as f64).log2();
        let forecast_coeff = (i as f64 * (log_size - 1.0) - 7.0).trunc() as i32; // coeff*(size/2)^i ~ 2^7 -> coeff ~ 2^-n? // coeff ~ 2^(7-i*(log2(size)-1)) // forecast max = 2^x(x-1)-7 // x = 8, max = 1785 < 2^11 // x = 16, max = 983033 < 2^20 // my_float32 s1e20f11

        let adjusted_coeff = *coeff*MyFp48::exp2(forecast_coeff);

        match adjusted_coeff.to_record_bytes() {
            Ok(record_f32) => out_vec.push(record_f32),
            Err("can't express f32, because of this MyFp48 abs is too small") => out_vec.push(0x0000),
            Err("can't express f32, because of this MyFp48 abs is too big") => {
                println!("can't express f32, because of this MyFp48 is too big");

                println!("returns record f32 max instead");
                let record_u16_max: u16 = 
                if adjusted_coeff.sign() == 1 { 0x7FFF }
                else { 0xFFFF };
                out_vec.push(record_u16_max);
            },
            _ => panic!("may can't see you"),
        }
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
    //println!("{:?}", comp);
    // if comp[0].is_nan() {println!("NaN!")}
    // else {
    //     println!("NOT NaN (T_T)");
    //     // println!("{:?}", comp)
    // }

}

#[test]
fn cmp() {

    let ele = MyFp48 { base: -0.000000000000000000000000000005458883, extra_exponent: 32768 };
    let log_size = (132 as f64).log2();
    let forecast_coeff = (0 as f64 * (log_size - 1.0) - 7.0).trunc() as i32;

    let hoge = ele*MyFp48::exp2(forecast_coeff); // -7
    let fuga = MyFp48::new(f32::EPSILON);
    if hoge < fuga {
        println!("smaller than f32 epsilon!, value is:{ele}");
        println!("{}\n{}",hoge.exponent(),fuga.exponent())
    }
}