mod raw;
pub use raw::*;

pub trait Poset {
  fn new(size: usize, relations: Vec<Vec<bool>>) -> Self;
  fn len(&self) -> usize;
  fn has_relation(&self, left: usize, right: usize) -> bool;
  fn verify_edges(&mut self, verifier: impl Fn(&Self, usize, usize) -> bool + std::marker::Sync + std::marker::Send) -> bool;
  fn replace(&mut self, node: usize, replacement: Self);
  fn report(&self);
}