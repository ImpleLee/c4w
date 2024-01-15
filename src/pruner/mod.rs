mod plain;
pub use plain::*;

use crate::states::*;

pub trait Pruner {
  fn prune_concrete<T: States>(states: ConcreteMappedStates<T>) -> (ConcreteMappedStates<T>, bool);
}
