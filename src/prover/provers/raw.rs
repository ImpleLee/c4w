use arrayvec::ArrayVec;
use itertools::Itertools;

use std::{collections::{HashSet, HashMap}, marker::PhantomData};
use crate::states::{States, GetNext, ConcreteMappedStates};
use super::{Poset, Branch, Next, Prover, WorkingProver};

struct WorkingRawProver<U: Poset, T: States> {
  poset: U,
  mapping: Vec<usize>,
  states: T
}
impl<U: Poset, T: States> WorkingRawProver<U, T> {
  fn get_next(&self, state: usize) -> ArrayVec<Vec<usize>, 7> {
    self.states.true_get_next(state, |v| {
      let mut result = vec![];
      for i in v.into_iter().map(|i| self.mapping[i]) {
        if result.iter().all(|&j| !self.poset.is_geq(j, i)) {
          result.push(i)
        }
      }
      result.sort_unstable();
      result
    })
  }
  fn split_nodes(&self, nexts: Vec<ArrayVec<Vec<usize>, 7>>) -> impl Iterator<Item=(usize, usize)> {
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
      nexts.into_iter()
        .map(|next| Next(next.into_iter().map(|next| branch_to_id[&next]).collect::<ArrayVec<_, 7>>()))
        .collect_vec()
    };
    let branches = branches.into_iter().map(|s| Branch(s)).collect_vec();
    let branch_geqs = branches.iter().flat_map(|left| {
      branches.iter().map(move |right| left.is_geq(right, |l, r| self.poset.is_geq(l, r)))
    }).collect::<Vec<_>>();
    let comperator = move |left: usize, right: usize| branch_geqs[left * branches_len + right];
    nexts.into_iter()
      .enumerate()
      .combinations(2)
      .filter_map(move |v| {
        let (left_id, left) = v[0].clone();
        let (right_id, right) = v[1].clone();
        if left.is_geq(&right, &comperator) {
          Some((left_id, right_id))
        } else if right.is_geq(&left, &comperator) {
          Some((right_id, left_id))
        } else {
          None
        }
      })
  }
}

impl<U: Poset, T: States> WorkingProver<T> for WorkingRawProver<U, T> {
  fn try_replace_node(&mut self) -> bool {
    eprintln!("try replace node");
    let mut prev_id_to_next = vec![HashMap::new(); self.poset.len()];
    let new_mapping = (0..self.states.len())
      .enumerate()
      .map(|(state, prev_id)| {
        let next = self.get_next(state);
        let new_id = prev_id_to_next[prev_id].len();
        *prev_id_to_next[prev_id].entry(next).or_insert(new_id)
      })
      .collect_vec();
    let poset_size_prev = self.poset.len();
    let deltas = prev_id_to_next.into_iter()
      .map(|nexts| {
        assert!(!nexts.is_empty());
        if nexts.len() == 1 {
          return None;
        }
        let mut nexts = nexts.into_iter().collect_vec();
        nexts.sort_unstable_by_key(|&(_, id)| id);
        assert!(nexts.iter().enumerate().all(|(i, (_, id))| i == *id));
        let nexts = nexts.into_iter()
          .map(|(next, _)| next)
          .collect_vec();
        let geqs = self.split_nodes(nexts).group_by(|&(left, _)| left);
        let mut geqs = geqs.into_iter().collect_vec();
        geqs.sort_unstable_by_key(|&(id, _)| id);
        let geqs = geqs.into_iter()
          .map(|(_, s)| s.map(|(_, j)| j).collect_vec())
          .collect_vec();
        Some(U::from_dag(geqs))
      })
      .collect_vec().into_iter()
      .enumerate()
      .map(|(node, replacement)| {
        let Some(replacement) = replacement else {
          return usize::MAX;
        };
        let delta = self.poset.len() - 1;
        self.poset.replace(node, replacement);
        delta
      })
      .collect_vec();
    if self.poset.len() == poset_size_prev {
      eprintln!("no node replacement");
      return false;
    }
    self.mapping.iter_mut()
      .zip(new_mapping.into_iter())
      .for_each(|(old, new)| {
        if new == 0 {
          return;
        }
        let delta = deltas[*old];
        assert_ne!(delta, usize::MAX);
        *old = new + delta;
      });
    eprintln!("node count after replacement: {}", self.poset.len());
    true
  }
  fn try_remove_edges(&mut self) -> bool {
    eprintln!("try remove edges");
    let false_edges = self.poset.get_reduction()
      .filter(|&(left, right)| {
        let left = self.get_next(left).into_iter()
          .map(|s| Branch(s))
          .collect::<ArrayVec<_, 7>>();
        let right = self.get_next(right).into_iter()
          .map(|s| Branch(s))
          .collect::<ArrayVec<_, 7>>();
        let left_next = Next((0..left.len()).collect());
        let right_next = Next((0..right.len()).collect());
        !left_next.is_geq(&right_next, |left_id, right_id| {
          left[left_id].is_geq(&right[right_id], |l, r| self.poset.is_geq(l, r))
        })
      })
      .collect_vec();
    if false_edges.is_empty() {
      eprintln!("no edge removal");
      return false;
    }
    eprintln!("remove {} edges", false_edges.len());
    false_edges.into_iter()
      .for_each(|(left, right)| self.poset.remove_edge(left, right));
    true
  }
  fn get_concrete(self) -> ConcreteMappedStates<T> {
    let nexts = (0..self.poset.len())
      .map(|i| self.get_next(i))
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
      poset: U::from_dag(vec![vec![]]),
      mapping: vec![0 as usize; states.len()],
      states
    }
  }
}