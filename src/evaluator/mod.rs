mod value_iteration;
pub use value_iteration::*;
mod loop_finder;


use crate::states::*;

pub trait Evaluator {
  type Item<'a> where Self: 'a;
  fn next<'a>(&'a mut self) -> Self::Item<'a>;
}