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

fn report<T: States>(minimized: &ConcreteMappedStates<T>) {
  eprintln!(
    "nodes: {}, edges: {}, original: {}",
    minimized.nexts.len(),
    minimized.nexts.continuations.len(),
    minimized.mapping.len()
  );
  let mut count_by_choices = vec![0; 7];
  for next in &minimized.nexts.cont_index {
    for &(begin, end) in next {
      count_by_choices[end - begin] += 1;
    }
  }
  eprintln!("count_by_choices: {:?}", count_by_choices);
}

fn main() {
  let continuations: HashMap<Field, HashMap<Piece, Vec<Field>>> =
    bincode::deserialize_from(std::io::stdin().lock()).unwrap();
  eprintln!("{}", continuations.len());

  let build_states = || FieldSequenceStates::<BagSequenceStates>::new(&continuations, 4, true);
  let num2state = build_states();
  eprintln!("{}", (&num2state).len());

  let mut minimized = ParallelMinimizer::minimize(num2state).concrete();
  report(&minimized);

  {
    while {
      let go ;
      (minimized, go) = PlainPruner::prune_concrete(minimized);
      go
    } {
      minimized = ParallelMinimizer::minimize(minimized).compose();
      report(&minimized);
    }
  }

  let proved = RawProver::<HierarchDAG>::prune(minimized);
  report(&proved);

  return;
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
