use rayon::prelude::*;
use sucds::bit_vectors::{BitVector, prelude::*};

use super::Poset;

pub struct BitVectorPoset {
  edges: Vec<BitVector>,
}

impl BitVectorPoset {
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

impl Poset for BitVectorPoset {
  fn new(size: usize, edges: Vec<Vec<bool>>) -> Self {
    assert!(size > 0);
    if size == 1 {
      return Self {
        edges: vec![BitVector::from_bit(true, 1)],
      }
    }
    let ret = Self {
      edges: edges.into_iter().map(BitVector::from_bits).collect(),
    };
    // ret.check();
    ret
  }
  fn report(&self) {
    eprintln!("poset nodes: {}", self.len());
  }
  fn len(&self) -> usize {
    self.edges.len()
  }
  fn has_relation(&self, left: usize, right: usize) -> bool {
    self.edges[left].access(right).unwrap()
  }
  fn replace(&mut self, node: usize, replacement: Self) {
    for (i, edge) in self.edges.iter_mut().enumerate() {
      if i == node {
        continue;
      }
      let connected = edge.access(node).unwrap();
      edge.extend(std::iter::repeat(connected).take(replacement.len() - 1));
    }
    let cloned = self.edges[node].clone();
    self.edges.extend(replacement.edges.iter().skip(1).map(|v| {
      let mut f = cloned.clone();
      f.set_bit(node, v.access(0).unwrap()).unwrap();
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
        BitVector::from_bits(v.iter()
          .enumerate()
          .map(|(j, connected)| {
            if !connected {
              return false;
            }
            verifier(self, i, j)
          }))
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
