use c4w::evaluator::*;
use c4w::states::*;
use c4w::minimizer::*;

use clap::Parser;
use crossterm::{cursor, terminal, queue, style, style::Stylize};
use std::io::{stderr, Write};
use indicatif::ProgressIterator;

/// Check the result of minimization by value iteration. 
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The path to the state file
    #[arg(long)]
    state_file: std::path::PathBuf,
}

const EPS: f64 = 1e-10;

fn value_iteration(minimized: &impl States) -> Vec<f64> {
  let mut stderr = stderr();
  let eps_str = format!("{:.5}", -EPS.log10());
  let mut evaluator = ValueIterator::new(minimized);
  loop {
    let (_, diff) = evaluator.next();
    queue!(stderr, cursor::MoveToColumn(0), terminal::Clear(crossterm::terminal::ClearType::CurrentLine)).unwrap();
    queue!(stderr, style::PrintStyledContent(format!("{:.5} / {}", -diff.log10(), eps_str).with(style::Color::Green))).unwrap();
    stderr.flush().unwrap();
    if diff < EPS {
      break;
    }
  }
  eprintln!();
  evaluator.values
}

fn main() {
  let args = Args::parse();
  let minimized: ConcreteMappedStates<FieldSequenceStates<BagSequenceStates>> =
    bincode::deserialize_from(std::fs::File::open(args.state_file).unwrap()).unwrap();

  eprintln!("minimized: nodes: {}, edges: {}, original: {}", minimized.nexts.len(), minimized.nexts.continuations.len(), minimized.mapping.len());
  let values1 = value_iteration(&minimized);

  let minimized2 = ParallelMinimizer::minimize(minimized.original.clone()).concrete();
  eprintln!("minimized2: nodes: {}, edges: {}, original: {}", minimized2.nexts.len(), minimized2.nexts.continuations.len(), minimized2.mapping.len());
  let values2 = value_iteration(&minimized2);

  let mut max_diff: f64 = 0.0;

  for i in (0..minimized.original.len()).progress() {
    let v1 = values1[minimized.mapping[i]];
    let v2 = values2[minimized2.mapping[i]];
    assert!((v1 - v2).abs() < EPS, "value mismatch at {}: {} vs {}", i, v1, v2);
    max_diff = max_diff.max((v1 - v2).abs());
  }

  eprintln!("values match");
  eprintln!("max diff: {}", max_diff);
}