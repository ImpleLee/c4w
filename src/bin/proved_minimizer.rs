use c4w::basics::*;
use c4w::evaluator::*;
use c4w::minimizer::*;

use c4w::prover::*;
use c4w::pruner::*;
use c4w::states::*;
use std::collections::HashMap;
use clap::Parser;
use bit_vec::BitVec;

fn report<T: States>(minimized: &ConcreteMappedStates<T>) {
  eprintln!(
    "nodes: {}, edges: {}, original: {}",
    minimized.nexts.len(),
    minimized.nexts.continuations.len(),
    minimized.mapping.len()
  );
  let mut count_by_choices = vec![0; 14];
  for next in &minimized.nexts.cont_index {
    for &(begin, end) in next {
      count_by_choices[end - begin] += 1;
    }
  }
  eprintln!("count_by_choices: {:?}", count_by_choices);
}

/// Simple program to minimize the states.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of previews
    #[arg(long)]
    preview: usize,

    /// Whether hold is enabled
    #[arg(long, action)]
    hold: bool,

    /// the path to the continuation file
    #[arg(long)]
    continuation: std::path::PathBuf,

    /// the path to save the result file
    #[arg(long)]
    output: Option<std::path::PathBuf>,
}

fn main() {
  let args = Args::parse();
  let continuations: HashMap<Field, HashMap<Piece, Vec<Field>>> =
    bincode::deserialize_from(std::fs::File::open(args.continuation).unwrap()).unwrap();
  eprintln!("{}", continuations.len());

  let build_states = || FieldSequenceStates::<BagSequenceStates>::new(&continuations, args.preview, args.hold);
  let num2state = build_states();
  eprintln!("{}", num2state.len());

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

  let proved = RawProver::<MatrixPoset<BitVec>>::prune(minimized);
  report(&proved);

  if let Some(output) = args.output {
    bincode::serialize_into(std::fs::File::create(output).unwrap(), &proved).unwrap();
  }
}
