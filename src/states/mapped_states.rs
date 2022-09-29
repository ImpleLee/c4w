use crate::states::*;
use rayon::prelude::*;
use std::marker::PhantomData;

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
    self.inverse.get(index).map(|&index| MappedState { index, _marker: PhantomData })
  }
  fn get_index(&self, state: &Self::State<'_>) -> Option<usize> {
    self.mapping.get(state.index).cloned()
  }
}

pub struct MappedState<'a, T: States> {
  index: usize,
  _marker: PhantomData<&'a T>
}

impl<'a, T: States> StateProxy<'a> for MappedState<'a, T> {
  type RealStates = MappedStates<T>;
  type Branch = Vec<usize>;
  type Proxy = Self;
  type BranchIter = arrayvec::IntoIter<Self::Branch, 7>;
  type SelfIter = Box<dyn Iterator<Item=Self::Proxy>+'a>;
  fn next_pieces(self, states: &'a Self::RealStates) -> Self::BranchIter {
    states.original.get_next(self.index, &*states.mapping).into_iter()
  }
  fn next_states(self, states: &'a Self::RealStates, piece: Self::Branch) -> Self::SelfIter {
    Box::new(
      piece.into_iter().map(|i| MappedState { index: states.inverse[i], _marker: PhantomData })
    )
  }
}
impl<'a, T: States> Clone for MappedState<'a, T> {
  fn clone(&self) -> Self {
    Self { ..*self }
  }
}
impl<'a, T: States> Copy for MappedState<'a, T> {}

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
