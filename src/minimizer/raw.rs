use crate::minimizer::*;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

// uses tons of memory
// not recommended unless you have a lot of memory
pub struct RawMinimizer {}

impl<T: States+std::marker::Sync+HasLength> Minimizer<T> for RawMinimizer {
  fn minimize(states: T) -> MinimizedStates<T> {
    let mut res = vec![0_usize; states.len()];
    let mut nexts = vec![];
    let mut last_length = 1;
    loop {
      let next_set =
        (0..res.len()).into_par_iter().map(|i| states.get_next(i, &res)).collect::<HashSet<_>>();
      if next_set.len() == last_length {
        break;
      }
      last_length = next_set.len();
      eprintln!("minimized states: {}", last_length);
      nexts = next_set.into_iter().collect::<Vec<_>>();
      let next_map =
        nexts.par_iter().enumerate().map(|(i, next)| (next, i)).collect::<HashMap<_, _>>();
      res = (0..res.len())
        .into_par_iter()
        .map(|i| next_map[&states.get_next(i, &res)])
        .collect::<Vec<_>>();
    }
    MinimizedStates { states, state2num: res, nexts: nexts.into_iter().collect() }
  }
}
