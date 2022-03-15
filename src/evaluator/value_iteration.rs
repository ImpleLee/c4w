use super::*;
use average::{Mean, Max, Estimate};
use rayon::prelude::*;
use ordered_float::NotNan;

pub struct ValueIterator<'a> {
  values: Vec<f64>,
  nexts: &'a Vec<Vec<Vec<usize>>>,
  epsilon: f64,
}

impl<'a> Evaluator<'a> for ValueIterator<'a> {
  fn new(next: &'a Vec<Vec<Vec<usize>>>, epsilon: f64) -> Self {
    Self {
      values: vec![0.0; next.len()],
      nexts: next,
      epsilon,
    }
  }
  fn get_values(&self) -> &Vec<f64> {
    &self.values
  }
}

impl<'a> Iterator for ValueIterator<'a> {
  type Item = f64;
  fn next(&mut self) -> Option<Self::Item> {
    let (new_values, diffs): (Vec<_>, Vec<_>) = (0..self.values.len()).into_par_iter().map(|j| {
      let mut value = Mean::new();
      for next in &self.nexts[j] {
        let mut this_value = Max::from_value(0.);
        for &k in next {
          this_value.add(self.values[k] + 1.);
        }
        value.add(this_value.max());
      }
      let new_value = value.mean();
      let old_value = self.values[j];
      let diff = NotNan::new((new_value - old_value).abs()).unwrap();
      (new_value, diff)
    }).unzip();
    let diff = diffs.iter().max().unwrap().into_inner();
    self.values = new_values;
    if diff < self.epsilon {
      None
    } else {
      Some(diff)
    }
  }
}