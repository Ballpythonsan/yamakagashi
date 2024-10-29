use std::{fmt, iter::Sum};

/// This lib provide VecTool. e.g. dotproduct, sqnorm, and hadamardproduct.
use crate::my_float::MyFp48;

pub trait VecTool {
    fn dot<'a, I>(&self, other:I) -> MyFp48 where I: DoubleEndedIterator<Item = &'a MyFp48>;
    fn sq_norm(&self) -> MyFp48;
}
impl VecTool for Vec<MyFp48> {
    fn dot<'a, I>(&self, other: I) -> MyFp48
    where I: DoubleEndedIterator<Item = &'a MyFp48> {
        self.iter().rev().zip(other.into_iter().rev()).map(|(&a, &b)| a * b).sum()
    }

    fn sq_norm(&self) -> MyFp48 {
        self.iter().rev().map(|&a| a * a).sum()
    }
}
impl VecTool for [MyFp48] {
    fn dot<'a, I>(&self, other: I) -> MyFp48 
    where I: DoubleEndedIterator<Item = &'a MyFp48> {
        self.iter().rev().zip(other.into_iter().rev()).map(|(&a, &b)| a * b).sum()
    }

    fn sq_norm(&self) -> MyFp48 {
        self.iter().rev().map(|&a| a * a).sum()
    }
}
impl VecTool for std::iter::Take<std::iter::Skip<std::iter::Take<std::iter::StepBy<std::iter::Skip<std::slice::Iter<'_, u8>>>>>> {
    fn dot<'a, I>(&self, other: I) -> MyFp48 
    where I: Iterator<Item = &'a MyFp48> {
        self.clone().zip(other.into_iter()).map(|(&a, &b)| (MyFp48::new(a as f32) * b)).sum()
    }

    fn sq_norm(&self) -> MyFp48 {
        self.clone().map(|a| {let _a = MyFp48::new(*a as f32); _a*_a}).sum()
    }
}
impl VecTool for std::iter::Take<std::iter::StepBy<std::iter::Skip<std::slice::Iter<'_, MyFp48>>>> {
    fn dot<'a, I>(&self, other: I) -> MyFp48 
    where I: DoubleEndedIterator<Item = &'a MyFp48> {
        self.clone().rev().zip(other.into_iter().rev()).map(|(&a, &b)| a * b).sum()
    }

    fn sq_norm(&self) -> MyFp48 {
        self.clone().rev().map(|&a| a*a).sum()
    }
}

impl Sum for MyFp48 {

    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(MyFp48::ZERO, |acc, ele| acc + ele)
    }
}

#[derive(Debug)]
pub struct DisplayVec<MyFp48>(pub Vec<MyFp48>);
impl fmt::Display for DisplayVec<MyFp48> {

    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // Vecの中身をカンマ区切りで表示する
        let formatted = self.0.iter()
            .map(|item| {
                if item.is_zero() {
                    if item.sign() == 1 {
                        format!("0 x 2^0")
                    } else {
                        format!("-0 x 2^0")
                    }
                } else {   
                    let mantissa = item.mantissa_and_sign();
                    let exponent = item.exponent();
                    
                    format!("{} x 2^{}", mantissa, exponent)
                }
            })
            .collect::<Vec<String>>()
            .join(", ");
        
        // 角括弧で囲んで表示
        write!(f, "[{}]", formatted)
    }
}

pub trait HadamardProduct {
    fn hadamard_product(&mut self, other: &Vec<MyFp48>);
}
impl HadamardProduct for Vec<MyFp48> {
    fn hadamard_product(&mut self, other: &Vec<MyFp48>) {
        self.iter_mut().zip(other.iter()).for_each(|(a, b)| *a *= *b);
    }
}