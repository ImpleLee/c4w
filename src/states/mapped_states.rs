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
  type State = usize;
  type Branch = Vec<usize>;
  fn decode(&self, index: usize) -> Option<Self::State> {
    Some(index)
  }
  fn encode(&self, state: &Self::State) -> Option<usize> {
    Some(*state)
  }
  fn next_pieces(&self, state: Self::State) -> impl Iterator<Item=Self::Branch> {
    self.original.get_next(self.inverse[state], &*self.mapping).into_iter()
  }
  fn next_states(&self, piece: Self::Branch) -> impl Iterator<Item=Self::State> {
    piece.into_iter()
  }
}

#[derive(Clone)]
pub struct ConcreteMappedStates<T: States> {
  pub original: T,
  pub mapping: Vec<usize>,
  pub nexts: Continuation
}

impl<T: States> States for ConcreteMappedStates<T> {
  type State = usize;
  type Branch = (usize, usize);
  fn encode(&self, state: &Self::State) -> Option<usize> {
    Some(*state)
  }
  fn decode(&self, index: usize) -> Option<Self::State> {
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

impl<T: States> ConcreteMappedStates<MappedStates<T>> {
  pub fn compose(mut self) -> ConcreteMappedStates<T> {
    self.original.mapping.par_iter_mut().for_each(|i| *i = self.mapping[*i]);
    ConcreteMappedStates {
      original: self.original.original,
      mapping: self.original.mapping,
      nexts: self.nexts
    }
  }
}

impl<T: States> ConcreteMappedStates<ConcreteMappedStates<T>> {
  pub fn compose(mut self) -> ConcreteMappedStates<T> {
    self.original.mapping.par_iter_mut().for_each(|i| *i = self.mapping[*i]);
    ConcreteMappedStates {
      original: self.original.original,
      mapping: self.original.mapping,
      nexts: self.nexts
    }
  }
}