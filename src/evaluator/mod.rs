mod value_iteration;
pub use value_iteration::*;
mod loop_finder;


use crate::states::*;

pub trait Evaluator<'a, T: States>: Iterator<Item=f64> {
  fn new(next: &'a T, epsilon: f64) -> Self;
  fn get_values(self) -> Vec<f64>;
}
