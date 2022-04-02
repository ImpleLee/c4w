use crate::states::*;
use crate::minimizer::*;
use rayon::prelude::*;
use std::iter::Iterator;
use std::collections::HashMap;

pub struct ParallelMinimizer {}

impl<T: States + std::marker::Sync + HasLength> Minimizer<T> for ParallelMinimizer {
  fn minimize(states: T) -> MinimizedStates<T> {
    let mut res = vec![0_usize; states.len()];
    res.shrink_to_fit();
    let mut seeds = vec![0];
    loop {
      let knowns = seeds.par_iter()
        .map(|&state| (states.get_next(state, &res), state))
        .collect::<HashMap<_, _>>();
      let mut new_res = vec![0_usize; states.len()];
      new_res.shrink_to_fit();
      let new_knowns = new_res.par_iter_mut()
        .enumerate()
        .fold(HashMap::new, |mut temp_knowns, (i, new_res)| {
          let next = states.get_next(i, &res);
          *new_res = match knowns.get(&next) {
            Some(&known) => known,
            None => temp_knowns.entry(next).or_insert(vec![i])[0],
          };
          temp_knowns
        })
        .reduce(HashMap::new, |mut knowns1, knowns2| {
          knowns2.into_iter().for_each(|(next, seeds)| {
            knowns1.entry(next).or_default().extend(seeds);
          });
          knowns1
        }).into_par_iter()
        .map(|(_, seeds)| seeds)
        .collect::<Vec<_>>();
      res = new_res;
      seeds.extend(new_knowns.iter().map(|seeds| seeds[0]));
      eprintln!("minimized states: {}", seeds.len());
      let seed_dedup = new_knowns.into_par_iter().flat_map(|seeds| {
        let seed0 = seeds[0];
        seeds.into_par_iter().skip(1).map(move |seed| (seed, seed0))
      }).collect::<HashMap<_, _>>();
      eprintln!("need to dedup: {}", seed_dedup.len());
      if seed_dedup.is_empty() {
        break
      }
      res.par_iter_mut().for_each(|res| {
        if let Some(&seed) = seed_dedup.get(res) {
          *res = seed;
        }
      });
    }
    let seed_map = seeds.par_iter().enumerate().map(|(i, &seed)| (seed, i)).collect::<HashMap<_, _>>();
    res.par_iter_mut().for_each(|res| {
      *res = seed_map[res];
    });
    let nexts = seeds.into_iter().map(|seed| states.get_next(seed, &res)).collect();
    MinimizedStates {
      states,
      state2num: res,
      nexts,
    }
  }
}