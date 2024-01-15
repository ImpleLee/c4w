pub trait Poset {
  fn from_dag(leqs: Vec<Vec<usize>>) -> Self;
  fn len(&self) -> usize;
  fn is_geq(&self, left: usize, right: usize) -> bool;
  fn get_reduction(&self) -> impl Iterator<Item=(usize, usize)>;
  fn remove_edge(&mut self, left: usize, right: usize);
  fn replace(&mut self, node: usize, replacement: Self);
}