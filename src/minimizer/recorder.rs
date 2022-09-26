use crate::minimizer::*;
use std::hash::Hash;

pub struct RecorderMinimizer {}

impl Minimizer for RecorderMinimizer {
  fn minimize<T: States>(states: T) -> MappedStates<T> {
    let mut mapping = vec![0_usize; states.len()];
    let mut recorder = Recorder::new();
    recorder.inverse.push(0);
    loop {
      recorder.mapping = recorder
        .inverse
        .par_iter()
        .enumerate()
        .map(|(i, &seed)| (states.get_next_id(seed, &*mapping), i))
        .collect();
      assert_eq!(recorder.len(), recorder.inverse.len());
      let mut new_mapping = vec![usize::MAX; states.len()];
      let news = new_mapping
        //.par_iter_mut()
        .iter_mut()
        .enumerate()
        .filter_map(|(i, num)| {
          let next = states.get_next_id(i, &*mapping);
          match recorder.find(&next) {
            Some(j) => {
              *num = j;
              None
            }
            None => Some(i)
          }
        })
        .collect_vec();
      eprintln!("unresolved: {}", news.len());
      if news.is_empty() {
        return MappedStates { original: states, mapping: new_mapping, inverse: recorder.inverse };
      }
      for i in news {
        let next = states.get_next_id(i, &*mapping);
        new_mapping[i] = recorder.record(next, i);
      }
      mapping = new_mapping;
      eprintln!("minimized states: {}", recorder.len());
      recorder.clear();
    }
  }
}

struct Recorder<T: Eq+Hash+Clone> {
  mapping: HashMap<T, usize>,
  inverse: Vec<usize>
}

impl<T: Eq+Hash+Clone> Recorder<T> {
  fn new() -> Self {
    Self { mapping: HashMap::new(), inverse: vec![] }
  }

  fn record(&mut self, state: T, position: usize) -> usize {
    if let Some(num) = self.mapping.get(&state) {
      *num
    } else {
      let num = self.len();
      self.inverse.push(position);
      self.mapping.insert(state, num);
      num
    }
  }

  fn find(&self, state: &T) -> Option<usize> {
    self.mapping.get(state).copied()
  }

  fn len(&self) -> usize {
    self.mapping.len()
  }

  fn clear(&mut self) {
    self.mapping.clear();
  }
}
