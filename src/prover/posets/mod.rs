mod raw;
pub use raw::*;

pub trait Poset {
  fn new(size: usize, relations: Vec<Vec<bool>>) -> Self;
  fn len(&self) -> usize;
  fn has_relation(&self, left: usize, right: usize) -> bool;
  fn verify_edges(&mut self, verifier: impl std::marker::Sync + std::marker::Send + Fn(&Self, usize, usize) -> bool) -> bool;
  fn replace(&mut self, node: usize, replacement: Self);
  fn report(&self);
}