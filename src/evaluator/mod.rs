mod value_iteration;
use crate::states::*;
pub use value_iteration::*;

pub trait Evaluator<T: States+HasLength>: Iterator<Item=f64> {
  fn new(next: T, epsilon: f64) -> Self;
  fn get_values(self) -> Vec<f64>;
}
