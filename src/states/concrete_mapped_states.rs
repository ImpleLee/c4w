use crate::states::*;

#[derive(Clone)]
pub struct ConcreteMappedStates<T: States> {
  pub original: T,
  pub mapping: Vec<usize>,
  pub nexts: Continuation
}

pub struct ConcreteMappedState {
  state: usize
}

impl<T: States> States for ConcreteMappedStates<T> {
  type State = ConcreteMappedState;
  type StateIter<'a> = Box<dyn Iterator<Item=Self::State>+'a> where T: 'a;
  type Branch = (usize, usize);
  type BranchIter<'a> = arrayvec::IntoIter<(usize, usize), 7> where T: 'a;
  fn get_index(&self, state: &Self::State) -> Option<usize> {
    Some(state.state)
  }
  fn get_state(&self, index: usize) -> Option<Self::State> {
    Some(ConcreteMappedState { state: index })
  }
  fn next_pieces(&self, state: Self::State) -> Self::BranchIter<'_> {
    self.nexts.cont_index[state.state].clone().into_iter()
  }
  fn next_states(&self, piece: Self::Branch) -> Self::StateIter<'_> {
    let (left, right) = piece;
    Box::new(
      self.nexts.continuations[left..right].iter().map(move |&state| ConcreteMappedState { state })
    )
  }
}

impl<T: States> HasLength for ConcreteMappedStates<T> {
  fn len(&self) -> usize {
    self.nexts.len()
  }
}
