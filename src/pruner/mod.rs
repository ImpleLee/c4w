mod plain;
pub use plain::*;

use crate::states::*;

// TODO idea: add a pruner to prove relationships based on inspiration from evaluation
// specifically, use evaluation for good proof initialization
// and then remove from both the input and the output pairs that does not prove
// BUT: why not just use every relationship as a startpoint?
pub trait Pruner<T: States> {
  fn prune(states: MappedStates<T>) -> ConcreteMappedStates<T>;
  fn prune_concrete(states: ConcreteMappedStates<T>) -> (ConcreteMappedStates<T>, bool);
}
