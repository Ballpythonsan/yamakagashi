/// base_exponent  is  8 bits
/// extra_exponent is 16 bits
/// exponent       is 24 bits
/// 
/// s1(u16)e8f23
/// u16*2^8+e8 - 2^23+1 is actual exponent

use std::{cmp::Ordering, fmt, ops};

#[derive(Clone, Copy)]
struct MyFp48 {

    // MyFp48 is a*2^exp * 2^(extension*2^8)
    // MyFp48s exp = uuuu_uuuu_uuuu_uuuu_eeee_eeee, exp - (2^23-1) is actually exponent
    base: f32,
    extra_exponent: u16,
}

impl MyFp48 {
    // create new MyFp48
    fn new(base: f32) -> Self {

        let base_exponent = ((base.to_bits() >> 23) & 0xFF) as i32 - ((1 << 7) - 1);
        let new_exponent = (base_exponent + (1 << 23) - 1) as u32;

        let new_base = f32::from_bits((base.to_bits() & 0b1000_0000_0111_1111_1111_1111_1111_1111) | ((new_exponent & 0b0000_0000_0000_0000_0000_0000_1111_1111) << 23));
        let new_extra_exponent = ((new_exponent & 0b0000_0000_1111_1111_1111_1111_0000_0000) >> 8) as u16;
        
        Self { base: new_base, extra_exponent: new_extra_exponent }
    }

    fn one() -> Self { Self { base: f32::from_bits(0x7F80_0000), extra_exponent: 0x7FFF }}
    
    fn zero() -> Self { Self { base: f32::from_bits(0x0), extra_exponent: 0x0 }}
    // create new MyFp48 exp2
    fn exp2(exponent: i32) -> Self {

        if (exponent < - (1 << 23)) | (((1 << 23) - 1) < exponent) { panic!("exp needs satisfy -(1 << 23) < exp < 2 << 23") }

        let new_exponent = (exponent + ((1 << 23) - 1)) as u32;
        
        let new_base = ((new_exponent & 0b0000_0000_0000_0000_0000_0000_1111_1111) << 23) as f32;
        let new_extra_exponent = ((new_exponent & 0b0000_0000_1111_1111_1111_1111_0000_0000) >> 8) as u16;
        
        Self { base: new_base, extra_exponent: new_extra_exponent }
    }

    fn is_nan(&self) -> bool { (self.exponent() == (1 << 23) - 1) & (self.mantissa().abs() == 1.0) }

    // get extended exponent
    fn exponent(&self) -> i32 {
        // no biased exponent

        let base_exponent = (self.base.to_bits() >> 23) & 0xFF;
        let exponent = ((self.extra_exponent as u32) << 8) | base_exponent;
        
        exponent as i32 - ((1 << 23) - 1)
    }

    // calc value
    fn value(&self) -> Result<f32, &str> {
 
        let exponent = self.exponent();
        let mantissa = self.mantissa();

        if exponent == 1 - (1 << 23) {
            if self.base.to_bits() & 0x007F_FFFF == 0x0 { Ok(f32::from_bits(self.base.to_bits() & 0x8000_0000)) } // 0 or -0
            else { Err("this MyFp48 is denormal number") }
        } else if exponent <= -127 {
            Err("can't express f32, because of this MyFp48 is too small")
        // } else if exponent == (1 << 25) - 1 - ((1 << 23) - 1) { 
        } else if 128 <= exponent {
            Err("can't express f32, because of this MyFp48 is too big")
        } else {
            let base_exponent = (exponent & 0xFF) as u32 + (1 << 7) - 1;
            Ok(f32::from_bits((mantissa.to_bits() & 0x807F_FFFF) | base_exponent << 23))
        }

    }

    // get sign
    fn sign(&self) -> i32 {
        if self.base.is_sign_negative() { -1 } else { 1 }
    }

    // get mantissa
    fn mantissa(&self) -> f32 {

        let base_bits = self.base.to_bits();
        let mantissa_bits = base_bits & 0x807F_FFFF;
        let mantissa = f32::from_bits(mantissa_bits | 0x3F80_0000);

        mantissa
    }

    // add
    fn add(self, other: Self) -> Self {

        let self_exponent = self.exponent();
        let other_exponent = other.exponent();

        let exponent_diff = (self_exponent - other_exponent).abs();

        if exponent_diff < 253 {

            let exponent_diff_s = exponent_diff as u32 & 0xFF;
            let new_diff1 = (127 + exponent_diff_s/2) << 23;
            let new_diff2 = (127 + exponent_diff_s/2 - exponent_diff_s) << 23;

            let adjusted_self_base=
                if self_exponent > other_exponent {f32::from_bits((self.base.to_bits() & 0x807F_FFFF) | new_diff1)}
                else {f32::from_bits((self.base.to_bits() & 0x807F_FFFF) | new_diff2)};
            
            let adjusted_other_base=
                if self_exponent < other_exponent {f32::from_bits((other.base.to_bits() & 0x807F_FFFF) | new_diff1)}
                else {f32::from_bits((other.base.to_bits() & 0x807F_FFFF) | new_diff2)};

            let temp_base = adjusted_self_base + adjusted_other_base;
            if temp_base.is_infinite() {panic!("add is infinite")}
            
            let diff_exponent = ((temp_base.to_bits() >> 23) & 0xFF) as i32 - (new_diff1 >> 23) as i32;
            let new_exponent: u32 = (self_exponent.max(other_exponent) + diff_exponent + (1 << 23) - 1) as u32;

            let new_base = f32::from_bits((temp_base.to_bits() & 0x807F_FFFF) | ((new_exponent & 0xFF) << 23));
            let new_extra_exponent = (new_exponent >> 8) as u16;

            Self { base: new_base, extra_exponent: new_extra_exponent }
        } else {
            println!("It's too big differense, \"add\" may failed");
            if self.extra_exponent < other.extra_exponent { other }
            else { self } // self.extra_exponent > other.extra_exponent
        }
    }
    
    // sub
    fn subtract(self, other: Self) -> Self {
        
        let self_exponent = self.exponent();
        let other_exponent = other.exponent();

        let exponent_diff = (self_exponent - other_exponent).abs();

        if exponent_diff < 253 {

            let exponent_diff_s = exponent_diff as u32 & 0xFF;
            let new_diff1 = (127 + exponent_diff_s/2) << 23;
            let new_diff2 = (127 + exponent_diff_s/2 - exponent_diff_s) << 23;

            let adjusted_self_base=
                if self_exponent > other_exponent {f32::from_bits((self.base.to_bits() & 0x807F_FFFF) | new_diff1)}
                else {f32::from_bits((self.base.to_bits() & 0x807F_FFFF) | new_diff2)};
            
            let adjusted_other_base=
                if self_exponent < other_exponent {f32::from_bits((other.base.to_bits() & 0x807F_FFFF) | new_diff1)}
                else {f32::from_bits((other.base.to_bits() & 0x807F_FFFF) | new_diff2)};

            let temp_base = adjusted_self_base - adjusted_other_base;
            if temp_base.is_infinite() {panic!("add is infinite")}
            
            let diff_exponent = ((temp_base.to_bits() >> 23) & 0xFF) as i32 - (new_diff1 >> 23) as i32;
            let new_exponent: u32 = (self_exponent.max(other_exponent) + diff_exponent + (1 << 23) - 1) as u32;

            let new_base = f32::from_bits((temp_base.to_bits() & 0x807F_FFFF) | ((new_exponent & 0xFF) << 23));
            let new_extra_exponent = (new_exponent >> 8) as u16;

            Self { base: new_base, extra_exponent: new_extra_exponent }
        } else {
            println!("It's too big differense, \"sub\" may failed");
            if self_exponent < other_exponent { Self { base: f32::from_bits(other.base.to_bits() ^ 0x8000_0000), extra_exponent: other.extra_exponent }} // -other
            else { self } // self.extra_exponent > other.extra_exponent
        }
    }

    // mul
    fn multiply(self, other: Self) -> Self {

        // a*2^exp * 2^(extension*2^8) * b*2^exp2 * 2^(extension2*2^8) = (a*2^exp)*(b*2^exp2)*2^((extension+extension2)*2^8)
        let temp_base = self.mantissa() * other.mantissa();
        if temp_base.is_nan() || temp_base.is_infinite() { panic!("base is Nan or infinity!, I'll write this pattern later") }

        let temp_base_exponent = ((temp_base.to_bits() >> 23) & 0xFF) as i32 - ((1 << 7) - 1); // todo
        let new_exponent = self.exponent() + other.exponent() + temp_base_exponent + ((1 << 23) - 1);

        let new_base =  f32::from_bits((temp_base.to_bits() & 0b1000_0000_0111_1111_1111_1111_1111_1111) | ((new_exponent as u32 & 0xFF) << 23));
        let new_extra_exponent = (new_exponent >> 8) as u16;
        
        Self { base: new_base, extra_exponent: new_extra_exponent } 
    }
    
    // dev
    fn divide(self, other: Self) -> Self {
        
        // a*2^exp * 2^(extension*2^8) / b*2^exp2 * 2^(extension2*2^8) = (a*2^exp)/(b*2^exp2)*2^((extension-extension2)*2^8)
        let temp_base = self.mantissa() / other.mantissa();
        if temp_base.is_nan() || temp_base.is_infinite() { panic!("base is Nan or infinity!, I'll write this pattern later") }

        let temp_base_exponent = ((temp_base.to_bits() >> 23) & 0xFF) as i32 - ((1 << 7) - 1); // todo 
        let new_exponent = self.exponent() - other.exponent() + temp_base_exponent + ((1 << 23) - 1);

        let new_base =  f32::from_bits((temp_base.to_bits() & 0b1000_0000_0111_1111_1111_1111_1111_1111) | (((new_exponent as u32) & 0xFF) << 23));
        let new_extra_exponent = (new_exponent >> 8) as u16;
        
        Self { base: new_base, extra_exponent: new_extra_exponent }
    }

}
// implemate Display
impl fmt::Display for MyFp48 {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        
        let mantissa = self.mantissa();
        let exponent = self.exponent();

        write!(f, "{} x 2^{}", mantissa, exponent)
    }
}

impl ops::Add for MyFp48 {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        self.add(rhs)
    }
}

impl ops::Sub for MyFp48 {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        self.subtract(rhs)
    }
}

impl ops::Mul for MyFp48 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        self.multiply(rhs)
    }
}

impl ops::Div for MyFp48 {
    type Output = Self;

    fn div(self, rhs: Self) -> Self::Output {
        self.divide(rhs)
    }
}

impl PartialEq for MyFp48 {

    fn eq(&self, other: &Self) -> bool {
        
        if self.is_nan() || other.is_nan() { return false; }
        
        self.base == other.base && self.extra_exponent == other.extra_exponent
    }
}

impl PartialOrd for MyFp48 {
    
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        
        if self.is_nan() || other.is_nan() { return None; }
        
        let self_exponent = self.exponent();
        let other_exponent = other.exponent();

        if self.sign() < other.sign() { return Some(Ordering::Less); }
        else if self.sign() > other.sign() { return Some(Ordering::Greater); }
        
        if self_exponent < other_exponent {
            if self.sign() == 1 { return Some(Ordering::Less); }
            else { return Some(Ordering::Greater); }
        }
        else if self_exponent > other_exponent {
            if self.sign() == 1 { return Some(Ordering::Greater); }
            else { return Some(Ordering::Less); }
        }

        self.mantissa().partial_cmp(&other.mantissa())
    }
}

#[test]
fn test_my_fp48_operations() {

    let a_ = -1.2;
    let b_ = 31.3;

    let a = MyFp48::new(a_);
    let b = MyFp48::new(b_);

    let sum_ = a_ + b_;
    let difference_ = a_ - b_;
    let product_ = a_ * b_;
    let quotient_ = a_ / b_;

    let sum = a + b;
    let difference = a - b;
    let product = a * b;
    let quotient = a / b;

    // println!("zero:       {}", MyFp48::zero().value().unwrap());
    // println!("one:        {}", MyFp48::one().value().unwrap());
    // println!("a:          {}", a.value().unwrap());
    // println!("b:          {}", b.value().unwrap());

    println!("a < b =     {}", a < b);
    println!("Sum:        {}, {}", sum, sum_);
    println!("Difference: {}, {}", difference, difference_);
    println!("Product:    {}, {}", product, product_);
    println!("Quotient:   {}, {}", quotient, quotient_);
}
