use crate::states::*;

#[derive(Clone)]
pub struct ConcreteMappedStates<T: States> {
  pub original: T,
  pub mapping: Vec<usize>,
  pub nexts: Continuation
}

impl<T: States> States for ConcreteMappedStates<T> {
  type State = usize;
  type Branch = (usize, usize);
  fn get_index(&self, state: &Self::State) -> Option<usize> {
    Some(*state)
  }
  fn get_state(&self, index: usize) -> Option<Self::State> {
    Some(index)
  }
  fn next_pieces(&self, state: Self::State) -> impl Iterator<Item=Self::Branch> {
    self.nexts.cont_index[state].iter().cloned()
  }
  fn next_states(&self, (left, right): Self::Branch) -> impl Iterator<Item=Self::State> {
    self.nexts.continuations[left..right].iter().cloned()
  }
}

impl<T: States> HasLength for ConcreteMappedStates<T> {
  fn len(&self) -> usize {
    self.nexts.len()
  }
}
