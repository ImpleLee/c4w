mod plain;
pub use plain::*;

use crate::minimizer::*;
use crate::states::*;

pub trait Pruner {
  fn prune<T: States+std::marker::Sync>(
    states: ConcreteMappedStates<T>
  ) -> (ConcreteMappedStates<T>, bool);
}
