use crate::minimizer::*;
use itertools::Itertools;
use rayon::prelude::*;
use std::iter::Iterator;

pub struct ConservMinimizer {}

impl Minimizer for ConservMinimizer {
  fn minimize<T: States+std::marker::Sync+HasLength>(states: T) -> MinimizedStates {
    let mut state_sorted = (0..states.len()).collect_vec();
    eprintln!("start sorting");
    state_sorted.par_sort_unstable_by_key(|&i| states.get_next_id(i, None));
    eprintln!("finish sorting");
    let state_order = state_sorted.clone();
    let (_, _, indices) = state_sorted.iter_mut().fold(
      (0, states.get_next_id(0, None), vec![0]),
      |(mut num, v, mut indices), i| {
        let next = states.get_next_id(*i, None);
        if next != v {
          num += 1;
          indices.push(*i);
        }
        *i = num;
        (num, next, indices)
      }
    );
    let mut state_order = state_order
      .par_iter()
      .zip(state_sorted.par_iter())
      .map(|(&i, &j)| (i, j))
      .collect::<Vec<_>>();
    state_order.par_sort_unstable_by_key(|&(i, _)| i);
    let state2num = state_order.par_iter().map(|&(_, j)| j).collect::<Vec<_>>();
    let nexts = indices.into_iter().map(|i| states.get_next(i, &*state2num)).collect();
    MinimizedStates { state2num, nexts }
  }
}
