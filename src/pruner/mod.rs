mod plain;
pub use plain::*;

use crate::minimizer::*;

pub trait Pruner {
  fn prune(states: MinimizedStates) -> (MinimizedStates, bool);
}
