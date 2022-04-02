use std::collections::{HashMap, HashSet};
use crate::states::*;
use crate::minimizer::*;
use rayon::prelude::*;

// uses tons of memory
// not recommended unless you have a lot of memory
pub struct RawMinimizer {}

impl<T: States + std::marker::Sync + HasLength> Minimizer<T> for RawMinimizer {
  fn minimize(states: T) -> MinimizedStates<T> {
    let mut res = vec![0_usize; states.len()];
    let mut nexts_v = vec![];
    let mut last_length = 1;
    loop {
      let get_next = |state: &T::State| -> Vec<Vec<usize>> {
        let mut nexts = Vec::new();
        for piece in state.next_pieces() {
          let mut next = Vec::new();
          for state in state.next_states(piece) {
            next.push(res[states.get_index(&state).unwrap()]);
          }
          next.sort_unstable();
          next.dedup();
          nexts.push(next);
        }
        if nexts.len() > 1 && nexts[1..].iter().all(|x| x == &nexts[0]) {
          nexts = vec![nexts[0].clone()];
        } else {
          nexts.sort();
        }
        nexts
      };
      let next_set = (0..res.len()).into_par_iter().map(|i| get_next(&states.get_state(i).unwrap())).collect::<HashSet<_>>();
      if next_set.len() == last_length {
        break;
      }
      last_length = next_set.len();
      eprintln!("minimized states: {}", last_length);
      nexts_v = next_set.into_iter().collect::<Vec<_>>();
      let next_map = nexts_v.par_iter().enumerate().map(|(i, next)| (next, i)).collect::<HashMap<_, _>>();
      res = (0..res.len()).into_par_iter().map(|i| next_map[&get_next(&states.get_state(i).unwrap())]).collect::<Vec<_>>();
    }
    let mut nexts = Continuation{ cont_index: vec![], continuations: vec![] };
    for next in nexts_v {
      nexts.add(next);
    }
    MinimizedStates {
      states,
      state2num: res,
      nexts,
    }
  }
}
