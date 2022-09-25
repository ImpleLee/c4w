use crate::minimizer::*;
use rayon::prelude::*;
use std::collections::{HashMap, HashSet};

// uses tons of memory
// not recommended unless you have a lot of memory
// possibly wrong at `nexts`, but partition is right
pub struct RawMinimizer {}

impl Minimizer for RawMinimizer {
  fn minimize<'a, T: States<'a>+std::marker::Sync+HasLength>(states: &'a T) -> MinimizedStates {
    let mut res = vec![0_usize; states.len()];
    let mut nexts;
    let mut last_length = 1;
    loop {
      let next_set = (0..res.len())
        .into_par_iter()
        .map(|i| states.get_next_id(i, &*res))
        .collect::<HashSet<_>>();
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
        .map(|i| next_map[&states.get_next_id(i, &*res)])
        .collect::<Vec<_>>();
    }
    let nexts = (0..last_length).into_iter().map(|seed| states.get_next(seed, &*res)).collect();
    MinimizedStates { state2num: res, nexts }
  }
}
