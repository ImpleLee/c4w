use crate::minimizer::*;
use ::dashmap::DashMap;
use rayon::prelude::*;
use std::collections::HashMap;

pub struct DashMapMinimizer {}

impl Minimizer for DashMapMinimizer {
  fn minimize<T: States+std::marker::Sync+HasLength>(states: T) -> MappedStates<T> {
    let mut res = vec![0_usize; states.len()];
    let mut seeds = vec![0];
    loop {
      let recorder = seeds
        .par_iter()
        .map(|&seed| {
          let next = states.get_next_id(seed, &*res);
          (next, seed)
        })
        .collect::<HashMap<_, _>>();
      let mut new_res = res.clone();
      let news = new_res
        .par_iter_mut()
        .enumerate()
        .filter_map(|(i, num)| {
          let next = states.get_next_id(i, &*res);
          match recorder.get(&next) {
            Some(r) => {
              *num = *r;
              None
            }
            None => Some((i, num))
          }
        })
        .collect::<Vec<_>>();
      eprintln!("unresolved: {}", news.len());
      if news.is_empty() {
        break;
      }
      let recorder = recorder.into_iter().collect::<DashMap<_, _>>();
      news.into_par_iter().for_each(|(i, num)| {
        let next = states.get_next_id(i, &*res);
        *num = *recorder.entry(next).or_insert(i).value();
      });
      res = new_res;
      seeds = recorder.into_iter().map(|(_, seed)| seed).collect::<Vec<_>>();
      eprintln!("minimized states: {}", seeds.len());
    }
    let next2num = seeds
      .par_iter()
      .enumerate()
      .map(|(i, &seed)| {
        let next = states.get_next_id(seed, &*res);
        (next, i)
      })
      .collect::<HashMap<_, _>>();
    res = (0..res.len())
      .into_par_iter()
      .map(|i| {
        let next = states.get_next_id(i, &*res);
        next2num[&next]
      })
      .collect();
    MappedStates { original: states, mapping: res, inverse: seeds }
  }
}
