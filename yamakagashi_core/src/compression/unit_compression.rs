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
// use crate::my_vector::DisplayVec;

// unit transform and compression

pub fn unit_compression(b: std::iter::Take<std::iter::Skip<std::iter::Take<std::iter::StepBy<std::iter::Skip<std::slice::Iter<'_, u8>>>>>>, quality: i32) -> Vec<u16> {
    let n: usize = b.len();
    let x:Vec<MyFp48> = (0..n).map(|i| MyFp48::new((-(n as i32)+1 + 2*i as i32) as f32 / 2.0)).collect(); // x == [(-n+1)/2, (-n+3)/2..(n-3)/2,(n-1)/2]
    let b_sq_norm = b.sq_norm();

    let mut a = vec![MyFp48::ZERO; n];
    let mut c = vec![MyFp48::ZERO; n];
    let mut l = vec![MyFp48::ZERO; n];
    let mut f = vec![MyFp48::ZERO; n];
    let mut g = vec![MyFp48::ZERO; n];
    let mut power_x = vec![MyFp48::ONE; n];

    let mut ac_even = MyFp48::ZERO;
    let mut ac_odd = MyFp48::ZERO;

    for i in 0..n {
        let m = i / 2;
        c[i] = b.dot(power_x.iter());
        l[i] = power_x.sq_norm();
        power_x.hadamard_product(&x);

        // make new f
        if i == 0 {f[0] = MyFp48::ONE / l[0];}
        else if i == 1 {g[0] = MyFp48::ONE / l[1];}
        else {
            if i % 2 == 0 {
                let error_f = l[m .. 2*m].dot(f.iter().take(m));
                let error_g = l[m+1 .. 2*m+1].dot(g.iter().take(m));
                let diff = error_f-error_g;
                assert!(!diff.is_zero(), "diff: {diff}");
                f.iter_mut().take(m+1).zip([MyFp48::ZERO].iter().chain(g.iter().take(m)))
                    .for_each(|(_f, &_g)| *_f = (*_f-_g)/diff );
            } else {
                let error_f = l[m+1 .. 2*m+2].dot(f.iter().take(m+1));
                let error_g = l[m+1 .. 2*m+1].dot(g.iter().take(m));
                let diff = error_g-error_f;
                assert!(!diff.is_zero(), "diff: {diff}");
                g.iter_mut().take(m+1).zip(f.iter().take(m+1))
                    .for_each(|(_g, &_f)| *_g = (*_g-_f)/diff );
            }
        }

        // make new a
        if i % 2 == 0 {
            let error_a = l[m .. 2*m].dot(a.iter().skip(0).step_by(2).take(m));
            let diff = c[i] - error_a;
            a.iter_mut().skip(0).step_by(2).take(m+1).zip(f.iter().take(m+1)).for_each(|(_a, &_f)| *_a += diff*_f );
            ac_even = a.iter().skip(0).step_by(2).take(m+1).dot(c.iter().skip(0).step_by(2).take(m+1));
        } else {
            let error_a = l[m+1 .. 2*m+1].dot(a.iter().skip(1).step_by(2).take(m));
            let diff = c[i] - error_a;
            a.iter_mut().skip(1).step_by(2).take(m+1).zip(g.iter().take(m+1)).for_each(|(_a, &_g)| *_a += diff*_g );
            ac_odd = a.iter().skip(1).step_by(2).take(m+1).dot(c.iter().skip(1).step_by(2).take(m+1));
        }
        // It can be reduced to this
        // let error_a = l[m+i%2 .. 2*m+i%2].dot(a.iter().skip(i%2).step_by(2).take(m));
        // let diff = c[i] - error_a;
        //     a.iter_mut().skip(i%2).step_by(2).take(m+1).zip( { if i%2 == 0 { &f } else { &g } }.iter().take(m+1) ).for_each(|(_a, &_fg)| *_a += diff*_fg );

        // quality check
        if b_sq_norm * MyFp48::new(quality as f32 / 100.0) < ac_even + ac_odd {
            return round_to_record_u16(a);
        }
    }

    // println!("quality isn't satisfy (T_T) final quality is: {:.3}", MyFp48::ONE - sse/ssd);
    return round_to_record_u16(a);
}

fn round_to_record_u16(vec:Vec<MyFp48>) -> Vec<u16> {

    let size = vec.len();
    let mut out_vec: Vec<u16> = Vec::with_capacity(size); // coeff*(size/2)^i ~ 2^7 -> coeff ~ 2^-n? // coeff ~ 2^(7-i*(log2(size)-1))

    // when is vec constant functions-coeffs
    let is_constant: bool = vec.iter().skip(1).fold(true, |acc, ele| acc & ele.is_zero() );
    if is_constant {

        let round_coeff = match vec[0].round_u8() {
            Ok(value) => {
                MyFp48::new(value as f32)
            },
            Err(_) => panic!("can't round u8, because too big"),
        };

        out_vec = vec![0; size];
        out_vec[0] = match round_coeff.to_record_bytes_with_forecast(-7) {
            Ok(record_f16) => record_f16,
            Err("can't express f16, because of this MyFp48 abs is too small") => 0x0000,
            Err("can't express f16, because of this MyFp48 abs is too big") => {
                println!("can't express f16, because of this MyFp48 abs is too big");

                println!("returns record f16 max instead");
                // let record_u16_max: u16 = 
                if round_coeff.sign() == 1 { 0x7FFF }
                else { 0xFFFF }
            },
            _ => panic!("may can't see you"),
        };

        return out_vec;
    }
    
    vec.iter().enumerate().for_each(|(i, coeff)| {

        let log_size = (size as f64).log2();
        let forecast_coeff = (i as f64 * (log_size - 1.0) - 7.0).trunc() as i32; // coeff*(size/2)^i ~ 2^7 -> coeff ~ 2^-n? // coeff ~ 2^(7-i*(log2(size)-1)) // forecast max = 2^x(x-1)-7 // x = 8, max = 1785 < 2^11 // x = 16, max = 983033 < 2^20 // my_float32 s1e20f11

        // let adjusted_coeff = *coeff*MyFp48::exp2(forecast_coeff);

        match coeff.to_record_bytes_with_forecast(forecast_coeff) {
            Ok(record_f16) => out_vec.push(record_f16),
            Err("can't express f16, because of this MyFp48 abs is too small") => out_vec.push(0x0000),
            Err("can't express f16, because of this MyFp48 abs is too big") => {
                println!("can't express f16, because of this MyFp48 abs is too big");
                if (coeff.exponent() == -172) && (forecast_coeff == 191) { 
                    println!("{:?}", vec.iter().map(|&ele| ele.exponent() ).collect::<Vec<_>>());
                    panic!("-172")
                }
                println!("returns record f16 max instead");
                let record_u16_max: u16 = 
                if coeff.sign() == 1 { 0x7FFF }
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
    [30, 32, 35, 32, 33, 32, 34, 35, 31, 28, 32, 29, 28, 33, 33, 30, 33, 34, 29, 31, 34, 29, 28, 29, 30, 32, 30, 28, 30, 28, 29, 32, 28, 30, 34, 30, 25, 30, 29, 28, 33, 29, 25, 32, 31, 28, 33, 30, 29, 28, 25, 28, 28, 28, 29, 34, 27, 26, 33, 30, 27, 32, 29, 27, 29, 27, 27, 31, 27, 25, 30, 31, 31, 31, 30, 29, 27, 26, 26, 26, 32, 30, 27, 29, 25, 23, 28, 31, 27, 24, 26, 23, 26, 30, 28, 24, 28, 28, 28, 28, 28, 27, 26, 27, 26, 25, 25, 26, 25, 28, 27, 25, 26, 29, 26, 25, 30, 26, 22, 25, 27, 27, 23, 23, 26, 24, 27, 25, 23, 26, 30, 26, 23, 24, 27, 26, 23, 28, 29, 26, 28, 27, 25, 24, 24, 28, 24, 24, 28, 22, 22, 26, 30, 27, 22, 24, 28, 27, 28, 25, 23, 25, 27, 27, 24, 21, 24, 26, 23, 23, 24, 22, 22, 23, 22, 23, 21, 24, 25, 21, 20, 23, 25, 23, 24, 21, 22, 23, 22, 22, 23, 23, 23, 23, 21, 22, 22, 23, 23, 22, 22, 23, 23, 22, 22, 22, 21, 23, 23, 25, 23, 21, 24, 24, 23, 25, 23, 22, 25, 25, 23, 24, 23, 22, 21, 23, 23, 22, 22, 24, 27, 23]
    ;
    let test_len = test_case.len();
    let test_iter: std::iter::Take<std::iter::Skip<std::iter::Take<std::iter::StepBy<std::iter::Skip<std::slice::Iter<'_, u8>>>>>> 
     = test_case.iter().skip(0).step_by(1).take(test_len).skip(0).take(test_len);

    let comp = unit_compression(test_iter, 85);
    println!("{:?}", comp);
    // let ans = 
    // [44585, 48348, 14250, 14013, 47434, 49898, 12976, 17532, 13318, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
    // ;
    // assert_eq!(comp, ans);
    // if comp[0].is_nan() {println!("NaN!")}
    // else {
    //     println!("NOT NaN (T_T)");
    //     // println!("{:?}", comp)
    // }

}

#[test]
fn cmp() {

    let ele = MyFp48 {base:2.11588983E-37, extra_exponent:32768};
    println!("{ele}");
    if ele < MyFp48::ZERO {
        println!("MyFp48::ZERO is bigger than ele");
    } else {
        println!("ele is bigger than MyFp48::ZERO");
    }
}

#[test]
fn record() {
    let mut vec = vec!
    [4, -3, -11, -13, -26, -27, -34, -34, -47, -45, -61, -59, -72, -71, -85, -85, -97, -97, -109, -112, -124, -128, -135, -138, -148, -148, -162, -164, -175, -172, -186, -188, -204, -198, -212, -213, -229, -225, -239, -240, -261, -253, -267, -269, -281, -281, -295, -294, -308, -307, -322, -322, -336, -335, -352, -353, -367, -366, -378, -381, -392, -392, -408, -408, -422, -420, -433, -436, -447, -447, -464, -463, -483, -477, -489, -496, -504, -503, -519, -520, -534, -532, -547, -547, -562, -564, -575, -575, -590, -590, -605, -605, -620, -620, -636, -639, -651, -651, -666, -669, -680, -680, -695, -695, -710, -709, -727, -728, -742, -740, -760, -760, -774, -775, -793, -790, -807, -805, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607, -8388607]
    ;
    let size = vec.len();
    for (i, expo) in vec.iter_mut().enumerate() {
        if *expo == -8388607 { *expo = 0; continue; }
        let log_size = (size as f64).log2();
        let forecast_coeff = (i as f64 * (log_size - 1.0) - 7.0).trunc() as i32;
        *expo += forecast_coeff;
    }
    println!("{:?}", vec);
}
