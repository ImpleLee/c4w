use crate::states::*;
use std::marker::PhantomData;

#[derive(Clone)]
pub struct ConcreteMappedStates<T: States> {
  pub original: T,
  pub mapping: Vec<usize>,
  pub nexts: Continuation
}

pub struct ConcreteMappedState<'s, T: States> {
  _marker: PhantomData<&'s ConcreteMappedStates<T>>,
  state: usize
}

impl<T: States> States for ConcreteMappedStates<T> {
  type State<'a> = ConcreteMappedState<'a, T> where T: 'a;
  fn get_index(&self, state: &Self::State<'_>) -> Option<usize> {
    Some(state.state)
  }
  fn get_state(&self, index: usize) -> Option<Self::State<'_>> {
    Some(ConcreteMappedState { state: index, _marker: PhantomData })
  }
}

impl<T: States> HasLength for ConcreteMappedStates<T> {
  fn len(&self) -> usize {
    self.nexts.len()
  }
}

impl<'a, T: States> StateProxy<'a> for ConcreteMappedState<'a, T> {
  type RealStates = ConcreteMappedStates<T>;
  type Branch = usize;
  type BranchIter = std::ops::Range<Self::Branch>;
  type SelfIter = Box<dyn Iterator<Item=Self>+'a>;
  fn next_pieces(self, states: &'a Self::RealStates) -> Self::BranchIter {
    0..states.nexts.cont_index[self.state].len()
  }
  fn next_states(self, states: &'a Self::RealStates, piece: Self::Branch) -> Self::SelfIter {
    let (left, right) = states.nexts.cont_index[self.state][piece];
    Box::new(
      states.nexts.continuations[left..right]
        .iter()
        .map(move |&state| ConcreteMappedState { state, ..self })
    )
  }
}
impl<'a, T: States> Clone for ConcreteMappedState<'a, T> {
  fn clone(&self) -> Self {
    Self { ..*self }
  }
}

impl<'a, T: States> Copy for ConcreteMappedState<'a, T> {}
