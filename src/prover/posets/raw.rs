use rayon::prelude::*;
use sucds::bit_vectors::{BitVector as OtherBitVector, prelude::*};

use super::Poset;

pub trait BoolVec:
    FromIterator<bool> + Extend<bool>
    + std::marker::Sync + std::marker::Send
    + Clone {
  fn len(&self) -> usize;
  fn get(&self, index: usize) -> Option<bool>;
  fn set(&mut self, index: usize, value: bool);
  fn iter(&self) -> impl Iterator<Item = bool> + '_;
}

pub struct MatrixPoset<V: BoolVec> {
  edges: Vec<V>,
}

impl<V: BoolVec> MatrixPoset<V> {
  fn check(&self) {
    for edges in self.edges.iter() {
      assert_eq!(self.len(), edges.len());
    }
    (0..self.len()).into_par_iter()
      .for_each(|i| {
        (0..self.len()).into_par_iter()
          .for_each(|j| {
            if i == j {
              assert!(self.has_relation(i, j));
            } else {
              assert!(!self.has_relation(i, j) || !self.has_relation(j, i));
              if !self.has_relation(i, j) {
                assert!((0..self.len()).all(|k| !self.has_relation(i, k) || !self.has_relation(k, j)));
              }
            }
          })
      });
  }
}

impl<V: BoolVec> Poset for MatrixPoset<V> {
  fn new(size: usize, edges: Vec<Vec<bool>>) -> Self {
    assert!(size > 0);
    if size == 1 {
      return Self {
        edges: vec![vec![true].into_iter().collect()],
      }
    }
    let ret = Self {
      edges: edges.into_iter().map(|v| v.into_iter().collect()).collect(),
    };
    ret.check();
    ret
  }
  fn report(&self) {
    eprintln!("poset nodes: {}", self.len());
  }
  fn len(&self) -> usize {
    self.edges.len()
  }
  fn has_relation(&self, left: usize, right: usize) -> bool {
    self.edges[left].get(right).unwrap()
  }
  fn replace(&mut self, node: usize, replacement: Self) {
    for (i, edge) in self.edges.iter_mut().enumerate() {
      if i == node {
        continue;
      }
      let connected = edge.get(node).unwrap();
      edge.extend(std::iter::repeat(connected).take(replacement.len() - 1));
    }
    let cloned = self.edges[node].clone();
    self.edges.extend(replacement.edges.iter().skip(1).map(|v| {
      let mut f = cloned.clone();
      f.set(node, v.get(0).unwrap());
      f.extend(v.iter().skip(1));
      f
    }));
    self.edges[node].extend(replacement.edges[0].iter().skip(1));
  }
  fn verify_edges(&mut self, verifier: impl Fn(&Self, usize, usize) -> bool + std::marker::Sync + std::marker::Send) -> bool {
    self.check();
    let mut checked_edges = self.edges.par_iter()
      .enumerate()
      .map(|(i, v)|
        (0..v.len())
          .map(|j| {
            if !v.get(j).unwrap() {
              return false;
            }
            verifier(self, i, j)
          })
          .collect::<V>()
      )
      .collect::<Vec<_>>();
    let len_changed_edges =  {
      std::mem::swap(&mut self.edges, &mut checked_edges);
      checked_edges.into_par_iter().zip(self.edges.par_iter())
        .map(|(edges, check_edges)| {
        edges.iter().zip(check_edges.iter())
          .filter(|&(connected, checked)| connected != checked)
          .count()
      })
      .sum::<usize>()
    };
    eprintln!("found {} internal false edges", len_changed_edges);
    self.check();
    len_changed_edges > 0
  }
}

impl BoolVec for Vec<bool> {
  fn len(&self) -> usize {
    self.len()
  }
  fn get(&self, index: usize) -> Option<bool> {
    self.as_slice().get(index).cloned()
  }
  fn set(&mut self, index: usize, value: bool) {
    self[index] = value;
  }
  fn iter(&self) -> impl Iterator<Item = bool> + '_ {
    self.as_slice().iter().cloned()
  }
}

#[derive(Clone)]
pub struct BitVector(OtherBitVector);

impl FromIterator<bool> for BitVector {
  fn from_iter<T: IntoIterator<Item = bool>>(iter: T) -> Self {
    Self(OtherBitVector::from_bits(iter))
  }
}
impl Extend<bool> for BitVector {
  fn extend<T: IntoIterator<Item = bool>>(&mut self, iter: T) {
    self.0.extend(iter);
  }
}

impl BoolVec for BitVector {
  fn len(&self) -> usize {
    self.0.len()
  }
  fn get(&self, index: usize) -> Option<bool> {
    self.0.access(index)
  }
  fn set(&mut self, index: usize, value: bool) {
    self.0.set_bit(index, value).unwrap();
  }
  fn iter(&self) -> impl Iterator<Item = bool> + '_ {
    self.0.iter()
  }
}