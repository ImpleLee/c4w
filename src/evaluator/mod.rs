mod value_iteration;
pub use value_iteration::*;

pub trait Evaluator<'a>: Iterator<Item=f64> {
  fn new(next: &'a Vec<Vec<Vec<usize>>>, epsilon: f64) -> Self;
  fn get_values(&self) -> &Vec<f64>;
}
