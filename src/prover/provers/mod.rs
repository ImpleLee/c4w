mod raw;
pub use raw::*;

use super::{ProvePruner, Branch, Next, Poset};
use crate::states::*;

trait WorkingProver<T: States> {
  fn try_replace_node(&mut self) -> bool;
  fn try_remove_edges(&mut self) -> bool;
  fn get_concrete(self) -> ConcreteMappedStates<T>;
}

trait Prover {
  fn new<T: States>(states: T) -> impl WorkingProver<T>;
}

impl<B: Prover> ProvePruner for B {
  fn prune<T: States>(states: ConcreteMappedStates<T>) -> ConcreteMappedStates<T> {
    let mut pruner = B::new(states);
    while pruner.try_replace_node() || pruner.try_remove_edges() {
    }
    pruner.get_concrete().compose()
  }
}