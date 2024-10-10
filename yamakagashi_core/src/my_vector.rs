use std::iter::Sum;

/// This lib provide VecTool. e.g. dotproduct, sqnorm, and hadamardproduct.
use crate::my_float::MyFp48;

pub trait VecTool {
    fn dot<'a, I>(&self, other:I) -> MyFp48 where I: IntoIterator<Item = &'a MyFp48>;
    fn sq_norm(&self) -> MyFp48;
}
impl VecTool for Vec<MyFp48> {
    fn dot<'a, I>(&self, other: I) -> MyFp48
    where I: IntoIterator<Item = &'a MyFp48> {
        self.iter().zip(other.into_iter()).map(|(&a, &b)| a * b).sum()
    }

    fn sq_norm(&self) -> MyFp48 {
        self.iter().map(|&a| a * a).sum()
    }
}
impl VecTool for [MyFp48] {
    fn dot<'a, I>(&self, other: I) -> MyFp48 
    where I: IntoIterator<Item = &'a MyFp48> {
        self.iter().zip(other.into_iter()).map(|(&a, &b)| a * b).sum()
    }

    fn sq_norm(&self) -> MyFp48 {
        self.iter().map(|&a| a * a).sum()
    }
}
impl VecTool for std::iter::Take<std::iter::Skip<std::iter::Take<std::iter::StepBy<std::iter::Skip<std::slice::Iter<'_, u8>>>>>> {
    fn dot<'a, I>(&self, other: I) -> MyFp48 
    where I: IntoIterator<Item = &'a MyFp48> {
        self.clone().zip(other.into_iter()).map(|(&a, &b)| (MyFp48::new(a as f32) * b)).sum()
    }

    fn sq_norm(&self) -> MyFp48 {
        self.clone().map(|a| {let _a = MyFp48::new(*a as f32); _a*_a}).sum()
    }
}

impl Sum for MyFp48 {

    fn sum<I: Iterator<Item = Self> + >(iter: I) -> Self {
        iter.fold(MyFp48::ZERO, |acc, ele| acc + ele)
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