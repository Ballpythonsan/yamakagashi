/// This lib provide VecTool. e.g. dotproduct, sqnorm, and hadamardproduct.

pub trait VecTool {
    fn dot<'a, I>(&self, other:I) -> f32 where I: IntoIterator<Item = &'a f32>;
    fn sq_norm(&self) -> f32;
}
impl VecTool for Vec<f32> {
    fn dot<'a, I>(&self, other: I) -> f32
    where I: IntoIterator<Item = &'a f32> {
        self.iter().zip(other.into_iter()).map(|(&a, b)| a * b).sum()
    }

    fn sq_norm(&self) -> f32 {
        self.iter().map(|&a| a * a).sum()
    }
}
impl VecTool for [f32] {
    fn dot<'a, I>(&self, other: I) -> f32 
    where I: IntoIterator<Item = &'a f32> {
        self.iter().zip(other.into_iter()).map(|(&a, b)| a * b).sum()
    }

    fn sq_norm(&self) -> f32 {
        self.iter().map(|&a| a * a).sum()
    }
}
impl VecTool for std::iter::Take<std::iter::Skip<std::iter::Take<std::iter::StepBy<std::iter::Skip<std::slice::Iter<'_, u8>>>>>> {
    fn dot<'a, I>(&self, other: I) -> f32 
    where I: IntoIterator<Item = &'a f32> {
        self.clone().zip(other.into_iter()).map(|(&a, b)| (a as f32 * b)).sum()
    }

    fn sq_norm(&self) -> f32 {
        self.clone().map(|a| (*a as f32)*(*a as f32)).sum()
    }
}

pub trait HadamardProduct {
    fn hadamard_product(&mut self, other: &Vec<f32>);
}
impl HadamardProduct for Vec<f32> {
    fn hadamard_product(&mut self, other: &Vec<f32>) {
        self.iter_mut().zip(other.iter()).for_each(|(a, b)| *a = *a * *b);
    }
}