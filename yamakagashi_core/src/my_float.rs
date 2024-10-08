/// base_exponent  is  8 bits
/// extra_exponent is 16 bits
/// exponent       is 24 bits
/// 
/// s1(u16)e8f23
/// u16*2^8+e8 - 2^23+1 is actual exponent

use std::{cmp::Ordering, fmt, ops};

const BASE_MANTISSA_AND_SIGN_MASK: u32 = 0x807F_FFFF;

#[derive(Clone, Copy, Debug)]
pub struct MyFp48 {

    // MyFp48 is a*2^exp * 2^(extension*2^8)
    // MyFp48s exp = uuuu_uuuu_uuuu_uuuu_eeee_eeee, exp - (2^23-1) is actually exponent
    pub base: f32,
    pub extra_exponent: u16,
}

impl MyFp48 {
    // create new MyFp48
    pub fn new(base: f32) -> Self {

        let base_exponent = ((base.to_bits() >> 23) & 0xFF) as i32 - ((1 << 7) - 1);
        let new_exponent = (base_exponent + (1 << 23) - 1) as u32;

        let new_base = f32::from_bits((base.to_bits() & BASE_MANTISSA_AND_SIGN_MASK) | ((new_exponent & 0xFF) << 23));
        let new_extra_exponent = (new_exponent >> 8) as u16;
        
        Self { base: new_base, extra_exponent: new_extra_exponent }
    }

    pub const ONE: MyFp48 = Self { base: f32::INFINITY, extra_exponent: 0x7FFF }; // Self { base: f32::from_bits(0x7F80_0000), extra_exponent: 0x7FFF }; // 0b0(0111_1111_1111_1111)111_1111_1000_0000_0000_0000_0000_0000
    
    pub const ZERO: MyFp48 = Self { base: 0f32, extra_exponent: 0x0 }; // Self { base: f32::from_bits(0x0), extra_exponent: 0x0 }; // 0b0(0000_0000_0000_0000)000_0000_0000_0000_0000_0000_0000_0000
    
    // pub const INFINITY: MyFp48 = Self { base: f32::INFINITY, extra_exponent: 0xFFFF};

    // pub const NEG_INFINITY: MyFp48 = Self { base: f32::NEG_INFINITY, extra_exponent: 0xFFFF};
    
    // pub fn default() -> Self { MyFp48::ZERO }
    // create new MyFp48 exp2
    pub fn exp2(exponent: i32) -> Self {

        if (exponent < - (1 << 23)) | (((1 << 23) - 1) < exponent) { panic!("exp needs satisfy -(1 << 23) < exp < 2 << 23") }

        let new_exponent = (exponent + ((1 << 23) - 1)) as u32;
        
        let new_base = f32::from_bits((new_exponent & 0xFF) << 23);
        let new_extra_exponent = ((new_exponent & 0xFF_FF00) >> 8) as u16;
        
        Self { base: new_base, extra_exponent: new_extra_exponent }
    }

    pub fn is_normal(&self) -> bool {
        let self_exponent = self.exponent();
        -(1 << 23) + 1 < self_exponent && self_exponent < (1 << 23)
    }

    pub fn is_zero(&self) -> bool { self.exponent() == -(1 << 23)+1 && self.mantissa_and_sign().abs() == 1.0 }

    pub fn round_u8(&self) -> Result<u8, ()> {

        let exponent = self.exponent();
        let mantissa = self.mantissa_and_sign();

        if exponent <= -127 { Ok(0u8) }
        else if 128 <= exponent { Err(()) }
        else {
            let f32_exponent = (exponent + (1 << 7)-1) as u32;
            let self_to_f32 = f32::from_bits((mantissa.to_bits() & BASE_MANTISSA_AND_SIGN_MASK) | (f32_exponent << 23));
            Ok(self_to_f32.round() as u8)
        }
    }
    // get extended exponent
    pub fn exponent(&self) -> i32 {
        // no biased exponent

        let base_exponent = (self.base.to_bits() >> 23) & 0xFF;
        let exponent = ((self.extra_exponent as u32) << 8) | base_exponent;
        
        exponent as i32 - (1 << 23) + 1
    }

    // calc value
    pub fn to_record_f32(&self) -> Result<f32, &str> {
 
        // out f32 havn't NaN and denormal number, but 0 is existing
        let exponent = self.exponent();
        let mantissa_and_sign = self.mantissa_and_sign();

        if self.is_zero() { Ok(0.0) }
        else if exponent <= -127 {
            Err("can't express f32, because of this MyFp48 is too small")
        // } else if exponent == (1 << 25) - 1 - ((1 << 23) - 1) { 
        } else if 128 <= exponent {
            Err("can't express f32, because of this MyFp48 is too big")
        } else {
            let base_exponent = (exponent + (1 << 7) - 1) as u32 & 0xFF;
            Ok(f32::from_bits((mantissa_and_sign.to_bits() & BASE_MANTISSA_AND_SIGN_MASK) | base_exponent << 23))
        }

    }

    pub fn from_record_f32(input: f32) -> Self {

        if input == 0.0 { return MyFp48::ZERO; }
        let new_exponent = ((input.to_bits() >> 23) & 0xFF) + (1 << 23) - (1 << 7);

        let new_base = f32::from_bits((input.to_bits() & BASE_MANTISSA_AND_SIGN_MASK) | ((new_exponent & 0xFF) << 23));
        let new_extra_exponent = (new_exponent >> 8) as u16;

        Self { base: new_base, extra_exponent: new_extra_exponent }
    }

    // get sign
    pub fn sign(&self) -> i32 {
        if self.base.is_sign_negative() { -1 } else { 1 }
    }

    // get mantissa
    pub fn mantissa_and_sign(&self) -> f32 {

        let base_bits = self.base.to_bits();
        let mantissa_bits = base_bits & BASE_MANTISSA_AND_SIGN_MASK;
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
                if self_exponent > other_exponent {f32::from_bits((self.base.to_bits() & BASE_MANTISSA_AND_SIGN_MASK) | new_diff1)}
                else {f32::from_bits((self.base.to_bits() & BASE_MANTISSA_AND_SIGN_MASK) | new_diff2)};
            
            let adjusted_other_base=
                if self_exponent < other_exponent {f32::from_bits((other.base.to_bits() & BASE_MANTISSA_AND_SIGN_MASK) | new_diff1)}
                else {f32::from_bits((other.base.to_bits() & BASE_MANTISSA_AND_SIGN_MASK) | new_diff2)};

            let temp_base = adjusted_self_base + adjusted_other_base;
            if temp_base.is_infinite() {panic!("add is infinite")}
            
            let diff_exponent = ((temp_base.to_bits() >> 23) & 0xFF) as i32 - (new_diff1 >> 23) as i32;
            let new_exponent: u32 = (self_exponent.max(other_exponent) + diff_exponent + (1 << 23) - 1) as u32;

            let new_base = f32::from_bits((temp_base.to_bits() & BASE_MANTISSA_AND_SIGN_MASK) | ((new_exponent & 0xFF) << 23));
            let new_extra_exponent = (new_exponent >> 8) as u16;

            Self { base: new_base, extra_exponent: new_extra_exponent }
        } else {
            // println!("It's too big differense, \"add\" may failed");
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
                if self_exponent > other_exponent {f32::from_bits((self.base.to_bits() & BASE_MANTISSA_AND_SIGN_MASK) | new_diff1)}
                else {f32::from_bits((self.base.to_bits() & BASE_MANTISSA_AND_SIGN_MASK) | new_diff2)};
            
            let adjusted_other_base=
                if self_exponent < other_exponent {f32::from_bits((other.base.to_bits() & BASE_MANTISSA_AND_SIGN_MASK) | new_diff1)}
                else {f32::from_bits((other.base.to_bits() & BASE_MANTISSA_AND_SIGN_MASK) | new_diff2)};

            let temp_base = adjusted_self_base - adjusted_other_base;
            if temp_base.is_infinite() {panic!("add is infinite")}
            
            let diff_exponent = ((temp_base.to_bits() >> 23) & 0xFF) as i32 - (new_diff1 >> 23) as i32;
            let new_exponent: u32 = (self_exponent.max(other_exponent) + diff_exponent + (1 << 23) - 1) as u32;

            let new_base = f32::from_bits((temp_base.to_bits() & BASE_MANTISSA_AND_SIGN_MASK) | ((new_exponent & 0xFF) << 23));
            let new_extra_exponent = (new_exponent >> 8) as u16;

            Self { base: new_base, extra_exponent: new_extra_exponent }
        } else {
            // println!("It's too big differense, \"sub\" may failed");
            if self_exponent < other_exponent { Self { base: f32::from_bits(other.base.to_bits() ^ 0x8000_0000), extra_exponent: other.extra_exponent }} // -other
            else { self } // self.extra_exponent > other.extra_exponent
        }
    }

    // mul
    fn multiply(self, other: Self) -> Self {

        if self.is_zero() || other.is_zero() { return Self::ZERO; }
        // a*2^exp * 2^(extension*2^8) * b*2^exp2 * 2^(extension2*2^8) = (a*2^exp)*(b*2^exp2)*2^((extension+extension2)*2^8)
        let temp_base = self.mantissa_and_sign() * other.mantissa_and_sign();
        if temp_base.is_nan() || temp_base.is_infinite() { panic!("base is Nan or infinity!, I'll write this pattern later") }

        let temp_base_exponent = ((temp_base.to_bits() >> 23) & 0xFF) as i32 - ((1 << 7) - 1); // todo
        let new_exponent = self.exponent() + other.exponent() + temp_base_exponent + ((1 << 23) - 1);

        let new_base =  f32::from_bits((temp_base.to_bits() & BASE_MANTISSA_AND_SIGN_MASK) | ((new_exponent as u32 & 0xFF) << 23));
        let new_extra_exponent = (new_exponent >> 8) as u16;
        
        Self { base: new_base, extra_exponent: new_extra_exponent } 
    }
    
    // dev
    fn divide(self, other: Self) -> Self {
        
        if other.is_zero() { panic!("div 0!") }
        if self.is_zero() { return self; }
        // a*2^exp * 2^(extension*2^8) / b*2^exp2 * 2^(extension2*2^8) = (a*2^exp)/(b*2^exp2)*2^((extension-extension2)*2^8)
        let temp_base = self.mantissa_and_sign() / other.mantissa_and_sign();
        if temp_base.is_nan() || temp_base.is_infinite() { panic!("base is Nan or infinity!, I'll write this pattern later") }

        let temp_base_exponent = ((temp_base.to_bits() >> 23) & 0xFF) as i32 - ((1 << 7) - 1); // todo 
        let new_exponent = self.exponent() - other.exponent() + temp_base_exponent + ((1 << 23) - 1);

        let new_base =  f32::from_bits((temp_base.to_bits() & BASE_MANTISSA_AND_SIGN_MASK) | (((new_exponent as u32) & 0xFF) << 23));
        let new_extra_exponent = (new_exponent >> 8) as u16;
        
        Self { base: new_base, extra_exponent: new_extra_exponent }
    }

}
// implemate Display
impl fmt::Display for MyFp48 {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        
        let mantissa = self.mantissa_and_sign();
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
        
        if self.is_normal() && other.is_normal() { return self.base == other.base && self.extra_exponent == other.extra_exponent; }
        else if self.is_zero() && other.is_zero() { return true; }
        else { return false; }
    }
}

impl PartialOrd for MyFp48 {
    
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        
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

        self.mantissa_and_sign().partial_cmp(&other.mantissa_and_sign())
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
