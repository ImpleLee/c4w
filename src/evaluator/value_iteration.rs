use super::*;
use average::{Estimate, Max, Merge};
use rayon::prelude::*;

pub struct ValueIterator<'a, T: States> {
  pub values: Vec<f64>,
  states: &'a T,
}

impl<'a, T: States> ValueIterator<'a, T> {
  pub fn new(states: &'a T) -> Self {
    let mut values = vec![0.0; states.len()];
    values.shrink_to_fit();
    Self { values, states }
  }
}

impl<'a, T: States> Evaluator for ValueIterator<'a, T> {
  type Item<'b> = (&'b [f64], f64) where Self: 'b;
  fn next<'b>(&'b mut self) -> Self::Item<'b> {
    let (new_values, diffs): (Vec<_>, MyMax) = (0..self.values.len())
      .into_par_iter()
      .map(|j| {
        let mut values = vec![];
        let mut counter_added = 0.;
        let state = self.states.decode(j).unwrap();
        for next in self.states.next_pieces(state) {
          let mut this_value = Max::from_value(0.);
          let mut added = false;
          for next_state in self.states.next_states(next) {
            this_value.add(self.values[self.states.encode(&next_state).unwrap()]);
            added = true;
          }
          values.push(this_value.max());
          if added {
            counter_added += 1.;
          }
        }
        values.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let new_value = if values.is_empty() {
          -0.0
        } else if values.iter().all(|&v| v == values[0]) {
          values[0] + (counter_added > 0.) as u8 as f64
        } else {
          (values.iter().sum::<f64>() + counter_added) / values.len() as f64
        };
        let old_value = self.values[j];
        let diff = (new_value - old_value).abs();
        (new_value, diff)
      })
      .unzip();
    let diff = MyMax::max(&diffs);
    self.values = new_values;
    self.values.shrink_to_fit();
    (&self.values, diff)
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
