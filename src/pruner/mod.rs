mod plain;
pub use plain::*;
mod prove;
pub use prove::*;

use crate::states::*;

// TODO idea: add a pruner to prove relationships based on inspiration from evaluation
// specifically, use evaluation for good proof initialization
// and then remove from both the input and the output pairs that does not prove
// BUT: why not just use every relationship as a startpoint?
pub trait Pruner {
  fn prune<T: States>(states: MappedStates<T>) -> ConcreteMappedStates<T>;
  fn prune_concrete<T: States>(states: ConcreteMappedStates<T>) -> (ConcreteMappedStates<T>, bool);
}
