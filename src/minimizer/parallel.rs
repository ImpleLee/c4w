use crate::states::*;
use crate::minimizer::*;
use rayon::prelude::*;
use std::iter::Iterator;
use std::collections::HashMap;

pub struct ParallelMinimizer {}

impl<T: States + std::marker::Sync + HasLength> Minimizer<T> for ParallelMinimizer {
  fn minimize(states: T) -> MinimizedStates<T> {
    let mut state2num = vec![0_usize; states.len()];
    state2num.shrink_to_fit();
    let mut seeds = vec![0];
    loop {
      let global = seeds.par_iter()
        .map(|&state| (states.get_next(state, &state2num), state))
        .collect::<HashMap<_, _>>();
      let mut new_state2num = vec![0_usize; states.len()];
      new_state2num.shrink_to_fit();
      let locals = new_state2num.par_iter_mut()
        .enumerate()
        .fold(HashMap::new, |mut local, (i, ret)| {
          let next = states.get_next(i, &state2num);
          *ret = match global.get(&next) {
            Some(&known) => known,
            None => local.entry(next).or_insert(vec![i])[0],
          };
          local
        }).reduce(HashMap::new, |mut local1, local2| {
          for (next, seeds) in local2 {
            local1.entry(next).or_default().extend(seeds);
          }
          local1
        }).into_par_iter()
        .map(|(_, seeds)| seeds)
        .collect::<Vec<_>>();
      state2num = new_state2num;
      seeds.extend(locals.iter().map(|seeds| seeds[0]));
      eprintln!("minimized states: {}", seeds.len());
      let seed_dedup = locals.into_par_iter().flat_map(|seeds| {
        let seed0 = seeds[0];
        seeds.into_par_iter().skip(1).map(move |seed| (seed, seed0))
      }).collect::<HashMap<_, _>>();
      if seed_dedup.is_empty() {
        break
      }
      eprintln!("need to dedup: {}", seed_dedup.len());
      state2num.par_iter_mut().for_each(|res| {
        if let Some(&seed) = seed_dedup.get(res) {
          *res = seed;
        }
      });
    }
    let seed_map = seeds.par_iter().enumerate()
      .map(|(i, &seed)| (seed, i))
      .collect::<HashMap<_, _>>();
    state2num.par_iter_mut().for_each(|res| {
      *res = seed_map[res];
    });
    let nexts = seeds.into_iter()
      .map(|seed| states.get_next(seed, &state2num))
      .collect();
    MinimizedStates {
      states,
      state2num,
      nexts,
    }
  }
}