use crate::states::*;

#[derive(Clone)]
pub struct ConcreteMappedStates<T: States> {
  pub original: T,
  pub mapping: Vec<usize>,
  pub nexts: Continuation
}

pub struct ConcreteMappedState<'s, T: States> {
  states: &'s ConcreteMappedStates<T>,
  state: usize
}

impl<T: States> States for ConcreteMappedStates<T> {
  type State<'a> = ConcreteMappedState<'a, T> where T: 'a;
  fn get_index(&self, state: &Self::State<'_>) -> Option<usize> {
    Some(state.state)
  }
  fn get_state(&self, index: usize) -> Option<Self::State<'_>> {
    Some(ConcreteMappedState { states: self, state: index })
  }
}

impl<T: States> HasLength for ConcreteMappedStates<T> {
  fn len(&self) -> usize {
    self.nexts.len()
  }
}

impl<'a, T: States> StateProxy for ConcreteMappedState<'a, T> {
  type Branch = usize;
  type BranchIter = std::ops::Range<Self::Branch>;
  type SelfIter = std::vec::IntoIter<Self>;
  fn next_pieces(&self) -> Self::BranchIter {
    0..self.states.nexts.cont_index[self.state].len()
  }
  fn next_states(&self, piece: Self::Branch) -> Self::SelfIter {
    let (left, right) = self.states.nexts.cont_index[self.state][piece];
    self.states.nexts.continuations[left..right]
      .iter()
      .map(|&state| ConcreteMappedState { state, ..*self })
      .collect_vec()
      .into_iter()
  }
}