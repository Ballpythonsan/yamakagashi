/// This lib provide VecTool. e.g. dotproduct, sqnorm, and hadamardproduct.

pub trait VecTool {
    fn dot<'a, I>(&self, other:I) -> f64 where I: IntoIterator<Item = &'a f64>;
    fn sq_norm(&self) -> f64;
}
impl VecTool for Vec<f64> {
    fn dot<'a, I>(&self, other: I) -> f64
    where I: IntoIterator<Item = &'a f64> {
        self.iter().zip(other.into_iter()).map(|(&a, b)| a * b).sum()
    }

    fn sq_norm(&self) -> f64 {
        self.iter().map(|&a| a * a).sum()
    }
}
impl VecTool for [f64] {
    fn dot<'a, I>(&self, other: I) -> f64 
    where I: IntoIterator<Item = &'a f64> {
        self.iter().zip(other.into_iter()).map(|(&a, b)| a * b).sum()
    }

    fn sq_norm(&self) -> f64 {
        self.iter().map(|&a| a * a).sum()
    }
}
impl VecTool for std::iter::Take<std::iter::Skip<std::iter::Take<std::iter::StepBy<std::iter::Skip<std::slice::Iter<'_, u8>>>>>> {
    fn dot<'a, I>(&self, other: I) -> f64 
    where I: IntoIterator<Item = &'a f64> {
        self.clone().zip(other.into_iter()).map(|(&a, b)| (a as f64 * b)).sum()
    }

    fn sq_norm(&self) -> f64 {
        self.clone().map(|a| (*a as f64)*(*a as f64)).sum()
    }
}

pub trait HadamardProduct {
    fn hadamard_product(&mut self, other: &Vec<f64>);
}
impl HadamardProduct for Vec<f64> {
    fn hadamard_product(&mut self, other: &Vec<f64>) {
        self.iter_mut().zip(other.iter()).for_each(|(a, b)| *a = *a * *b);
    }
}