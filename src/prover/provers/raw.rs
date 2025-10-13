use arrayvec::ArrayVec;
use itertools::Itertools;
use rayon::prelude::*;
use indicatif::ProgressIterator;

use std::{collections::{HashSet, HashMap}, marker::PhantomData};
use crate::states::{States, GetNext, ConcreteMappedStates};
use super::{Poset, Branch, Next, Prover, WorkingProver};

struct WorkingRawProver<U: Poset, T: States> {
  poset: U,
  mapping: Vec<usize>,
  seeds: Vec<usize>,
  states: T
}
impl<U: Poset, T: States> WorkingRawProver<U, T> {
  fn static_get_next(poset: &U, mapping: &[usize], states: &T, state: usize) -> ArrayVec<Branch, 7> {
    states.true_get_next(state, |v| {
      let mut result = vec![];
      for i in v.into_iter().map(|i| mapping[i]) {
        result.retain(|&j| !poset.has_relation(i, j));
        if result.iter().all(|&j| !poset.has_relation(j, i)) {
          result.push(i)
        }
      }
      result.sort_unstable();
      Branch(result)
    })
  }
  fn get_next(&self, state: usize) -> ArrayVec<Branch, 7> {
    Self::static_get_next(&self.poset, &self.mapping, &self.states, state)
  }
  fn split_nodes(&self, nexts: Vec<ArrayVec<Branch, 7>>) -> Vec<Vec<bool>> {
    let branches = nexts.iter()
      .flatten()
      .collect::<HashSet<_>>()
      .into_iter()
      .cloned()
      .collect::<Vec<_>>();
    let branches_len = branches.len();
    let nexts = {
      let branch_to_id = branches.iter()
        .enumerate()
        .map(|(i, next)| (next.clone(), i))
        .collect::<HashMap<_, _>>();
      nexts.into_par_iter()
        .map(|next| Next(next.into_iter().map(|next| branch_to_id[&next]).collect::<ArrayVec<_, 7>>()))
        .collect::<Vec<_>>()
    };
    let branch_geqs = branches.iter().flat_map(|left| {
      branches.iter().map(move |right| left.is_geq(right, |l, r| self.poset.has_relation(l, r)))
    }).collect::<Vec<_>>();
    let comperator = move |left: usize, right: usize| branch_geqs[left * branches_len + right];
    nexts.par_iter()
      .map(
        |next_i| nexts.par_iter().map(|next_j| next_i.is_geq(next_j, &comperator)).collect()
      )
      .collect()
  }
}

impl<U: Poset, T: States> WorkingProver<T> for WorkingRawProver<U, T> {
  fn try_replace_node(&mut self) -> bool {
    eprint!("try replacing node: ");
    let prev_nexts = self.seeds.iter()
      .map(|&i| self.get_next(i))
      .collect_vec();
    let mut prev_id_to_next = vec![Vec::<(ArrayVec<Branch, 7>, usize)>::new(); self.poset.len()];
    let new_mapping = self.mapping.iter()
      .enumerate()
      .map(|(state, &prev_id)| {
        let next = self.get_next(state);
        if next == prev_nexts[prev_id] {
          return 0;
        }
        for (i, known_next) in prev_id_to_next[prev_id].iter().enumerate() {
          if next == known_next.0 {
            return i+1;
          }
        }
        let new_id = prev_id_to_next[prev_id].len()+1;
        prev_id_to_next[prev_id].push((next, state));
        new_id
      })
      .collect_vec();
    let largest_new_dag = prev_id_to_next.iter().map(|m| m.len()).max().unwrap() + 1;
    if largest_new_dag > 1 {
      eprintln!("largest new dag: {}", largest_new_dag);
      self.seeds.extend(prev_id_to_next.iter().flat_map(|v| v.iter().map(|&(_, i)| i)));
    } else if largest_new_dag == 1 {
      eprintln!("no node replacement");
    }
    let deltas = prev_id_to_next.iter()
      .map(|nexts| nexts.len())
      .scan(self.poset.len(), |acc, new_len| {
        let value = *acc;
        *acc += new_len;
        Some(value)
      })
      .collect_vec();
    prev_id_to_next.into_iter()
      .progress()
      .enumerate()
      .for_each(|(node, v)| {
        if v.is_empty() {
          return;
        }
        self.poset.replace(node, U::new(
          v.len() + 1,
          self.split_nodes(
            std::iter::once(prev_nexts[node].clone()).chain(v.into_iter().map(|(next, _)| next)).collect()
          )
        ));
      });
    if largest_new_dag == 1 {
      return false;
    }
    (&mut self.mapping, &new_mapping).into_par_iter()
      .for_each(|(old, &new)| {
        if new == 0 {
          return;
        }
        let delta = deltas[*old];
        assert_ne!(delta, usize::MAX);
        *old = new - 1 + delta;
      });
    eprint!("=> ");
    self.poset.report();
    true
  }
  fn try_remove_edges(&mut self) -> bool {
    eprint!("try remove edges: ");
    let found = self.poset.verify_edges(|poset, left, right| {
      let left = Self::static_get_next(poset, &self.mapping, &self.states, self.seeds[left]);
      let right = Self::static_get_next(poset, &self.mapping, &self.states, self.seeds[right]);
      let left_next = Next((0..left.len()).collect());
      let right_next = Next((0..right.len()).map(|i| i + left.len()).collect());
      
      left_next.is_geq(&right_next, |left_id, right_id| {
        left[left_id].is_geq(&right[right_id - left.len()], |l, r| poset.has_relation(l, r))
      })
    });
    eprint!("=> ");
    self.poset.report();
    found
  }
  fn get_concrete(self) -> ConcreteMappedStates<T> {
    let nexts = self.seeds.iter()
      .map(|&i| self.get_next(i).into_iter().map(|s| s.0))
      .collect();
    ConcreteMappedStates {
      original: self.states,
      mapping: self.mapping,
      nexts
    }
  }
}

pub struct RawProver<U: Poset>(PhantomData<U>);
impl<U: Poset> Prover for RawProver<U> {
  fn new<T: States>(states: T) -> impl WorkingProver<T> {
    WorkingRawProver {
      poset: U::new(1, vec![vec![true]]),
      mapping: vec![0_usize; states.len()],
      seeds: vec![0],
      states
    }
  }
}