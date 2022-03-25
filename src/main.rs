mod basics;
mod states;
mod minimizer;
mod evaluator;
mod printer;

use basics::*;
use states::*;
use minimizer::*;
use evaluator::*;
use printer::*;
use std::collections::HashMap;

fn main() {
  let continuations: HashMap<Field, HashMap<Piece, Vec<Field>>> = bincode::deserialize_from(
    std::io::stdin().lock(),
  ).unwrap();
  eprintln!("{}", continuations.len());

  let num2state = RandomStates::new(&continuations, 5, true);
  eprintln!("{}", (&num2state).len());

  let minimized = RecorderMinimizer::minimize(&num2state);

  let mut last_diff: f64 = 1.;
  const EPS: f64 = 1e-10;
  let mut evaluator = ValueIterator::new(&minimized, EPS);
  for (i, diff) in evaluator.by_ref().enumerate() {
    let expected = (diff.log10() - EPS.log10()) / (last_diff.log10() - diff.log10());
    eprintln!("{}/{:.2}: {}", i, expected + i as f64, diff);
    last_diff = diff;
  }

  let values = evaluator.get_values();
  // printer::MarkovAverage::print(&field2state, &values, &num2state);
}
