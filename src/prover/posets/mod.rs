pub trait Poset {
  fn new(size: usize, geqs: impl Iterator<Item=(usize, usize)>) -> Self;
  fn len(&self) -> usize;
  fn is_geq(&self, left: usize, right: usize) -> bool;
  fn get_reduction(&self) -> impl Iterator<Item=(usize, usize)>;
  fn remove_edge(&mut self, left: usize, right: usize);
  fn replace(&mut self, node: usize, replacement: Self);
}