use super::*;
use average::{Estimate, Max, Mean, Merge};
use rayon::prelude::*;

pub struct ValueIterator<'a, T: States> {
  values: Vec<f64>,
  states: &'a T,
  epsilon: f64
}

impl<'a, T: States> Evaluator<'a, T> for ValueIterator<'a, T> {
  fn new(next: &'a T, epsilon: f64) -> Self {
    let mut values = vec![0.0; next.len()];
    values.shrink_to_fit();
    Self { values, states: next, epsilon }
  }
  fn get_values(self) -> Vec<f64> {
    self.values
  }
}

impl<'a, T: States> Iterator for ValueIterator<'a, T> {
  type Item = f64;
  fn next(&mut self) -> Option<Self::Item> {
    let (new_values, diffs): (Vec<_>, MyMax) = (0..self.values.len())
      .into_par_iter()
      .map(|j| {
        let mut value = Mean::new();
        let state = self.states.get_state(j).unwrap();
        for next in state.next_pieces() {
          let mut this_value = Max::from_value(0.);
          for next_state in state.next_states(next) {
            this_value.add(self.values[self.states.get_index(&next_state).unwrap()] + 1.);
          }
          value.add(this_value.max());
        }
        let new_value = value.mean();
        let old_value = self.values[j];
        let diff = (new_value - old_value).abs();
        (new_value, diff)
      })
      .unzip();
    let diff = diffs.max();
    self.values = new_values;
    self.values.shrink_to_fit();
    if diff < self.epsilon {
      None
    } else {
      Some(diff)
    }
  }
}

#[derive(Default)]
struct MyMax(Max);

impl MyMax {
  fn max(&self) -> f64 {
    self.0.max()
  }
}

impl Estimate for MyMax {
  fn add(&mut self, other: f64) {
    self.0.add(other);
  }
  fn estimate(&self) -> f64 {
    self.0.estimate()
  }
}

impl Merge for MyMax {
  fn merge(&mut self, other: &Self) {
    self.0.merge(&other.0);
  }
}

impl ParallelExtend<f64> for MyMax {
  fn par_extend<I: IntoParallelIterator<Item=f64>>(&mut self, par_iter: I) {
    self.merge(
      &par_iter
        .into_par_iter()
        .fold(MyMax::default, |mut acc, x| {
          acc.add(x);
          acc
        })
        .reduce(MyMax::default, |mut acc, x| {
          acc.merge(&x);
          acc
        })
    );
  }
}
