use crate::states::*;

#[derive(Clone)]
pub struct ConcreteMappedStates<T: States> {
  pub original: T,
  pub mapping: Vec<usize>,
  pub nexts: Continuation
}

impl<T: States> States for ConcreteMappedStates<T> {
  type State = usize;
  type StateIter<'a> = std::iter::Cloned<std::slice::Iter<'a, Self::State>> where Self: 'a;
  type Branch = (usize, usize);
  type BranchIter<'a> = std::iter::Cloned<std::slice::Iter<'a, Self::Branch>> where Self: 'a;
  fn get_index(&self, state: &Self::State) -> Option<usize> {
    Some(*state)
  }
  fn get_state(&self, index: usize) -> Option<Self::State> {
    Some(index)
  }
  fn next_pieces(&self, state: Self::State) -> Self::BranchIter<'_> {
    self.nexts.cont_index[state].iter().cloned()
  }
  fn next_states(&self, (left, right): Self::Branch) -> Self::StateIter<'_> {
    self.nexts.continuations[left..right].iter().cloned()
  }
}

impl<T: States> HasLength for ConcreteMappedStates<T> {
  fn len(&self) -> usize {
    self.nexts.len()
  }
}
