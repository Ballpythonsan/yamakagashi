/// This lib provide VecTool. e.g. dotproduct, sqnorm, and hadamardproduct.

use num::rational::Ratio;

pub trait VecTool {
    fn dot<'a, I>(&self, other:I) -> Ratio<i32> where I: IntoIterator<Item = &'a Ratio<i32>>;
    fn sq_norm(&self) -> Ratio<i32>;
}
impl VecTool for Vec<Ratio<i32>> {
    fn dot<'a, I>(&self, other: I) -> Ratio<i32>
    where I: IntoIterator<Item = &'a Ratio<i32>> {
        self.iter().zip(other.into_iter()).map(|(&a, b)| Ratio::from(a) * b).sum()
    }

    fn sq_norm(&self) -> Ratio<i32> {
        self.iter().map(|&a| a * a).sum()
    }
}
impl VecTool for [Ratio<i32>] {
    fn dot<'a, I>(&self, other: I) -> Ratio<i32> 
    where I: IntoIterator<Item = &'a Ratio<i32>> {
        self.iter().zip(other.into_iter()).map(|(&a, b)| a * b).sum()
    }

    fn sq_norm(&self) -> Ratio<i32> {
        self.iter().map(|&a| a * a).sum()
    }
}
impl VecTool for std::iter::Take<std::iter::Skip<std::iter::Take<std::iter::StepBy<std::iter::Skip<std::slice::Iter<'_, u8>>>>>> {
    fn dot<'a, I>(&self, other: I) -> Ratio<i32> 
    where I: IntoIterator<Item = &'a Ratio<i32>> {
        self.clone().zip(other.into_iter()).map(|(&a, b)| (Ratio::from_integer(a as i32) * b)).sum()
    }

    fn sq_norm(&self) -> Ratio<i32> {
        self.clone().map(|a| Ratio::from_integer(*a as i32).pow(2)).sum()
    }
}

pub trait HadamardProduct {
    fn hadamard_product(&mut self, other: &Vec<Ratio<i32>>);
}
impl HadamardProduct for Vec<Ratio<i32>> {
    fn hadamard_product(&mut self, other: &Vec<Ratio<i32>>) {
        self.iter_mut().zip(other.iter()).for_each(|(a, b)| *a = *a * *b);
    }
}