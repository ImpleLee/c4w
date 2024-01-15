mod basics;
mod evaluator;
mod minimizer;
mod printer;
mod prover;
mod pruner;
mod states;

use basics::*;
use evaluator::*;
use minimizer::*;
use printer::*;
use prover::*;
use pruner::*;
use states::*;
use std::collections::HashMap;

fn main() {
  let continuations: HashMap<Field, HashMap<Piece, Vec<Field>>> =
    bincode::deserialize_from(std::io::stdin().lock()).unwrap();
  eprintln!("{}", continuations.len());

  let build_states = || FieldSequenceStates::<BagSequenceStates>::new(&continuations, 6, true);
  let num2state = build_states();
  eprintln!("{}", (&num2state).len());

  {
    let num2state = build_states();
    eprintln!(
      "has infinite loop: {}",
      LoopFinder::has_loop(ParallelMinimizer::minimize(num2state).concrete())
    );
  }

  let mut minimized = ParallelMinimizer::minimize(num2state).concrete();

  loop {
    eprintln!(
      "nodes: {}, edges: {}, original: {}",
      minimized.nexts.len(),
      minimized.nexts.continuations.len(),
      minimized.mapping.len()
    );
    let merged: bool;
    (minimized, merged) = PlainPruner::prune_concrete(minimized);
    if !merged {
      break;
    }
    minimized = ParallelMinimizer::minimize(minimized).compose();
  }

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
