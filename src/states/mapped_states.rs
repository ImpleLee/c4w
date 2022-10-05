use crate::states::*;
use rayon::prelude::*;

pub struct MappedStates<T: States> {
  pub original: T,
  pub mapping: Vec<usize>,
  pub inverse: Vec<usize>
}

impl<T: States> HasLength for MappedStates<T> {
  fn len(&self) -> usize {
    self.inverse.len()
  }
}

impl<T: States> States for MappedStates<T> {
  type State = MappedState;
  type StateIter<'a> = Box<dyn Iterator<Item=Self::State>+'a> where T: 'a;
  type Branch = Vec<usize>;
  type BranchIter<'a> = arrayvec::IntoIter<Self::Branch, 7> where T: 'a;
  fn get_state(&self, index: usize) -> Option<Self::State> {
    self.inverse.get(index).map(|&index| MappedState { index })
  }
  fn get_index(&self, state: &Self::State) -> Option<usize> {
    self.mapping.get(state.index).cloned()
  }
  fn next_pieces(&self, state: Self::State) -> Self::BranchIter<'_> {
    self.original.get_next(state.index, &*self.mapping).into_iter()
  }
  fn next_states(&self, piece: Self::Branch) -> Self::StateIter<'_> {
    Box::new(piece.into_iter().map(|i| MappedState { index: self.inverse[i] }))
  }
}

pub struct MappedState {
  index: usize
}

impl<T: States> MappedStates<MappedStates<T>> {
  pub fn compose(mut self) -> MappedStates<T> {
    self.original.mapping.par_iter_mut().for_each(|i| *i = self.mapping[*i]);
    self.inverse.par_iter_mut().for_each(|i| *i = self.original.inverse[*i]);
    self.original.inverse = self.inverse;
    self.original
  }
}

impl<T: States> MappedStates<T> {
  pub fn concrete(self) -> ConcreteMappedStates<T> {
    let nexts =
      self.inverse.into_iter().map(|i| self.original.get_next(i, &*self.mapping)).collect();
    ConcreteMappedStates { original: self.original, mapping: self.mapping, nexts }
  }
}

impl<T: States> MappedStates<ConcreteMappedStates<T>> {
  // TODO: sort by self.inverse first to remap current mapping
  // so that self.original.nexts can be incrementally changed without extra memory overhead
  pub fn compose(mut self) -> ConcreteMappedStates<T> {
    self.original.nexts =
      self.inverse.into_iter().map(|i| self.original.get_next(i, &*self.mapping)).collect();
    self.original.mapping.par_iter_mut().for_each(|i| *i = self.mapping[*i]);
    self.original
  }
}
