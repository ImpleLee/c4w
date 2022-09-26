use crate::minimizer::*;
use std::collections::HashSet;

// uses tons of memory
// not recommended unless you have a lot of memory
// possibly wrong at `nexts`, but partition is right
pub struct RawMinimizer {}

impl Minimizer for RawMinimizer {
  fn minimize<T: States>(states: T) -> MappedStates<T> {
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
      nexts = next_set.into_iter().collect_vec();
      let next_map =
        nexts.par_iter().enumerate().map(|(i, next)| (next.clone(), i)).collect::<HashMap<_, _>>();
      res = (0..res.len())
        .into_par_iter()
        .map(|i| next_map[&states.get_next_id(i, &*res)])
        .collect::<Vec<_>>();
    }
    let mut inverse = res.par_iter().cloned().enumerate().collect::<Vec<_>>();
    inverse.par_sort_unstable_by_key(|&(_, to)| to);
    inverse.dedup_by_key(|&mut (_, to)| to);
    MappedStates {
      original: states,
      mapping: res,
      inverse: inverse.into_par_iter().map(|(from, _)| from).collect()
    }
  }
}
