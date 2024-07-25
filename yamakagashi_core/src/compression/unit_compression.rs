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

use crate::my_vector::{VecTool, HadamardProduct};
use num::{rational::Ratio, integer::lcm};

// unit transform and compression

pub fn unit_compression(b: std::iter::Take<std::iter::Skip<std::iter::Take<std::iter::StepBy<std::iter::Skip<std::slice::Iter<'_, u8>>>>>>, quality: i32) -> (i32, Vec<i32>) {
    let n: usize = b.len();
    let x:Vec<Ratio<i32>> = if n%2 == 0{(0..n).map(|i| Ratio::new(-(n as i32)+1 + 2*i as i32, 2)).collect::<Vec<Ratio<i32>>>()}
            else{(0..n).map(|i| Ratio::from_integer((-(n as i32)+1)/2 + i as i32)).collect::<Vec<Ratio<i32>>>()}; // x == [(-n+1)/2, (-n+3)/2..(n-3)/2,(n-1)/2]
    let b_sq_norm = b.sq_norm();

    let mut a = vec![Ratio::ZERO; n];
    let mut c = vec![Ratio::ZERO; n];
    let mut l = vec![Ratio::ZERO; n];
    let mut f = vec![Ratio::ZERO; n];
    let mut power_x = vec![Ratio::ONE; n];

    let mut error_1 = Ratio::ZERO;
    let mut error_2 = Ratio::ZERO;
    let mut error_a = Ratio::ZERO;

    let mut ssd = Ratio::ZERO;
    let mut sse = Ratio::ZERO;

    for i in 0..n {
        let m = i / 2;
        c[i] = b.dot(&power_x);
        l[i] = power_x.sq_norm();
        power_x.hadamard_product(&x);

        // make new f
        if i == 0 {f[0] = Ratio::ONE / l[0];}
        else if i == 1 {f[1] = Ratio::ONE / l[1];}
        else {
            if i % 2 == 0 {
                error_1 = l[m..2 * m].dot(f.iter().skip(0).step_by(2).take(m));
                error_2 = l[m + 1..2 * m + 1].dot(f.iter().skip(1).step_by(2).take(m));
                f.iter_mut().skip(0).step_by(2).take(m).for_each(|a| *a /= error_1-error_2);
                for j in 0..m {
                    let _temp = f[1 + 2 * j];
                    f[2 + 2 * j] -= _temp / (error_1-error_2);
                }
            } else {
                error_1 = l[m + 1..2 * m + 1].dot(f.iter().skip(0).step_by(2).take(m + 1));
                // error_2 = &l[m   : 2*m].dot(&f.iter().skip(0).step_by(2).take(m));
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
            if ssd <= Ratio::from_integer(5) {return coeffs_organize(a);}
        } else if is_quality_satisfy(quality, sse, ssd) {return coeffs_organize(a);}
    }
    return coeffs_organize(a);
}

fn is_quality_satisfy(quality: i32, sse: Ratio<i32>, ssd:Ratio<i32>) -> bool {
    Ratio::new(quality, 100) <= Ratio::ONE - ssd / sse // quality <= 1 - SSE/SSD
}

fn coeffs_organize(old_coeffs: Vec<Ratio<i32>>) -> (i32, Vec<i32>) {
    let mut new_coeffs = vec![0; old_coeffs.len()];
    // let denom_lcm = old_coeffs.iter().map(|a| *a.denom()).reduce(|acc, a| lcm(acc, a)).unwrap();
    let denom_lcm = old_coeffs.iter().map(|a| *a.denom()).reduce(|acc, a| lcm(acc, a)).expect("denom over flow?");
    new_coeffs.iter_mut().zip(old_coeffs).for_each(|(new, old)| *new = old.numer()*(denom_lcm/old.denom()));
    (denom_lcm, new_coeffs)
}