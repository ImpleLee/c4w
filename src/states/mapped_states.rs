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
  type State<'a> = MappedState<'a, T> where T: 'a;
  fn get_state(&self, index: usize) -> Option<Self::State<'_>> {
    self.inverse.get(index).map(|&index| MappedState { states: &self, index })
  }
  fn get_index(&self, state: &Self::State<'_>) -> Option<usize> {
    self.mapping.get(state.index).cloned()
  }
}

pub struct MappedState<'a, T: States> {
  states: &'a MappedStates<T>,
  index: usize
}

impl<'a, T: States> StateProxy for MappedState<'a, T> {
  type Branch = Vec<usize>;
  type Proxy = Self;
  type BranchIter = arrayvec::IntoIter<Self::Branch, 7>;
  type SelfIter = std::vec::IntoIter<Self>;
  fn next_pieces(&self) -> Self::BranchIter {
    self.states.original.get_next(self.index, &*self.states.mapping).into_iter()
  }
  fn next_states(&self, piece: Self::Branch) -> Self::SelfIter {
    piece
      .into_iter()
      .map(|i| MappedState { states: self.states, index: self.states.inverse[i] })
      .collect_vec()
      .into_iter()
  }
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
