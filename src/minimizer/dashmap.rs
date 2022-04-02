use crate::states::*;
use crate::minimizer::*;
use ::dashmap::DashMap;
use std::collections::HashMap;
use rayon::prelude::*;

pub struct DashMapMinimizer {}

impl<T: States + std::marker::Sync + HasLength> Minimizer<T> for DashMapMinimizer {
  fn minimize(states: T) -> MinimizedStates<T> {
    let mut res = vec![0_usize; states.len()];
    let get_next = |state: &T::State, res: &Vec<usize>| -> Vec<Vec<usize>> {
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
    let mut seeds = vec![0];
    loop {
      let recorder = seeds.par_iter().map(|&seed| {
        let next = get_next(&states.get_state(seed).unwrap(), &res);
        (next, seed)
      }).collect::<HashMap<_, _>>();
      let mut new_res = res.clone();
      let news = new_res.par_iter_mut().enumerate().filter_map(|(i, num)| {
        let next = get_next(&states.get_state(i).unwrap(), &res);
        match recorder.get(&next) {
          Some(r) => {
            *num = *r;
            None
          }
          None => Some((i, num))
        }
      }).collect::<Vec<_>>();
      eprintln!("unresolved: {}", news.len());
      if news.is_empty() {
        break;
      }
      let recorder = recorder.into_iter().collect::<DashMap<_, _>>();
      news.into_par_iter().for_each(|(i, num)| {
        let next = get_next(&states.get_state(i).unwrap(), &res);
        *num = *recorder.entry(next).or_insert(i).value();
      });
      res = new_res;
      seeds = recorder.into_iter().map(|(_, seed)| seed).collect::<Vec<_>>();
      eprintln!("minimized states: {}", seeds.len());
    }
    let next2num = seeds.par_iter().enumerate().map(|(i, &seed)| {
      let next = get_next(&states.get_state(seed).unwrap(), &res);
      (next, i)
    }).collect::<HashMap<_, _>>();
    res = (0..res.len()).into_par_iter().map(|i| {
      let next = get_next(&states.get_state(i).unwrap(), &res);
      next2num[&next]
    }).collect();
    let mut nexts = Continuation{ cont_index: vec![], continuations: vec![] };
    for &seed in seeds.iter() {
      let next = get_next(&states.get_state(seed).unwrap(), &res);
      nexts.add(next);
    }
    MinimizedStates {
      states,
      state2num: res,
      nexts,
    }
  }
}
