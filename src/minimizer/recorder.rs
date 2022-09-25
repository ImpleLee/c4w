use crate::minimizer::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::hash::Hash;

pub struct RecorderMinimizer {}

impl Minimizer for RecorderMinimizer {
  fn minimize<T: States+std::marker::Sync+HasLength>(states: T) -> MappedStates<T> {
    let mut res = vec![0_usize; states.len()];
    let mut recorder = Recorder::new();
    recorder.seeds.push(0);
    loop {
      recorder.state2num = recorder
        .seeds
        .par_iter()
        .enumerate()
        .map(|(i, &seed)| (states.get_next_id(seed, &*res), i))
        .collect();
      assert_eq!(recorder.len(), recorder.seeds.len());
      let mut new_res = vec![usize::MAX; states.len()];
      let news = new_res
        //.par_iter_mut()
        .iter_mut()
        .enumerate()
        .filter_map(|(i, num)| {
          let next = states.get_next_id(i, &*res);
          match recorder.find(&next) {
            Some(j) => {
              *num = j;
              None
            }
            None => Some(i)
          }
        })
        .collect::<Vec<_>>();
      eprintln!("unresolved: {}", news.len());
      if news.is_empty() {
        return MappedStates { original: states, mapping: new_res, inverse: recorder.seeds }
      }
      for i in news {
        let next = states.get_next_id(i, &*res);
        new_res[i] = recorder.record(next, i);
      }
      res = new_res;
      eprintln!("minimized states: {}", recorder.len());
      recorder.clear();
    }
  }
}

struct Recorder<T: Eq+Hash+Clone> {
  state2num: HashMap<T, usize>,
  seeds: Vec<usize>
}

impl<T: Eq+Hash+Clone> Recorder<T> {
  fn new() -> Self {
    Self { state2num: HashMap::new(), seeds: vec![] }
  }

  fn record(&mut self, state: T, position: usize) -> usize {
    if let Some(num) = self.state2num.get(&state) {
      *num
    } else {
      let num = self.len();
      self.seeds.push(position);
      self.state2num.insert(state, num);
      num
    }
  }

  fn find(&self, state: &T) -> Option<usize> {
    self.state2num.get(state).copied()
  }

  fn len(&self) -> usize {
    self.state2num.len()
  }

  fn clear(&mut self) {
    self.state2num.clear();
  }
}
