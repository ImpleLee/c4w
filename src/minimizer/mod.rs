use std::hash::Hash;
use std::collections::HashMap;
use crate::states::*;
use rayon::prelude::*;


struct Recorder<T> where T: Eq + Hash + Clone {
  num2state: Vec<T>,
  state2num: HashMap<T, usize>,
  seeds: Vec<usize>,
}

impl<T> Recorder<T> where T: Eq + Hash + Clone {
  fn new() -> Self {
    Self{ num2state: vec![], state2num: HashMap::new(), seeds: vec![] }
  }
  
  fn record(&mut self, state: T, position: usize) -> usize {
    if let Some(num) = self.state2num.get(&state) {
      *num
    } else {
      let num = self.num2state.len();
      self.num2state.push(state.clone());
      self.seeds.push(position);
      self.state2num.insert(state, num);
      num
    }
  }
  
  fn find(&self, state: &T) -> Option<usize> {
    self.state2num.get(state).map(|&num| num)
  }
  
  fn len(&self) -> usize {
    self.num2state.len()
  }
  
  fn clear(&mut self) {
    self.num2state.clear();
    self.state2num.clear();
  }
}

pub fn minimize_states<T: States + std::marker::Sync + HasLength>(states: T) -> (Vec<usize>, Vec<Vec<Vec<usize>>>) {
  let mut last_count = 1;
  let mut res = vec![0 as usize; states.len()];
  let mut recorder = Recorder::new();
  recorder.seeds.push(0);
  loop {
    let get_next = |state: &T::State| -> Vec<Vec<usize>> {
      let mut nexts = Vec::new();
      for piece in state.next_pieces() {
        let mut next = Vec::new();
        for state in state.next_states(piece) {
          next.push(res[states.get_index(&state).unwrap()]);
        }
        next.sort();
        next.dedup();
        nexts.push(next);
      }
      nexts.sort();
      nexts
    };
    recorder.num2state = recorder.seeds.par_iter().map(|&seed| get_next(&states.get_state(seed).unwrap())).collect();
    recorder.state2num = recorder.num2state.iter().enumerate().map(|(i, s)| (s.clone(), i)).collect();
    assert_eq!(recorder.num2state.len(), recorder.state2num.len());
    let mut new_res = vec![usize::MAX; states.len()];
    let news = new_res.iter_mut().enumerate().par_bridge().filter_map(|(i, num)| {
      let next = get_next(&states.get_state(i).unwrap());
      match recorder.find(&next) {
        Some(j) => {
          *num = j;
          None
        }
        None => Some(i)
      }
    }).collect::<Vec<_>>();
    eprintln!("unresolved: {}", news.len());
    for i in news {
      let next = get_next(&states.get_state(i).unwrap());
      new_res[i] = recorder.record(next, i);
    }
    res = new_res;
    eprintln!("minimized states: {}", recorder.len());
    if recorder.len() == last_count {
      return (res, recorder.num2state)
    }
    last_count = recorder.len();
    recorder.clear();
  }
}