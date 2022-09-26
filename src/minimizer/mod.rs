use crate::states::*;
mod dashmap;
pub use self::dashmap::*;
mod recorder;
pub use recorder::*;
mod raw;
pub use raw::*;
mod parallel;
pub use parallel::*;
mod conservative;
use arrayvec::ArrayVec;
pub use conservative::*;
use gcd::Gcd;
use rayon::prelude::*;

pub trait Minimizer {
  fn minimize<T: States+std::marker::Sync+HasLength>(states: T) -> MappedStates<T>;
}

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
  type BranchIter = arrayvec::IntoIter<Vec<usize>, 7>;
  type SelfIter = std::vec::IntoIter<Self>;
  fn next_pieces(&self) -> Self::BranchIter {
    self.states.original.get_next(self.index, &*self.states.mapping).into_iter()
  }
  fn next_states(&self, piece: Self::Branch) -> Self::SelfIter {
    piece
      .into_iter()
      .map(|i| MappedState { states: self.states, index: self.states.inverse[i] })
      .collect::<Vec<_>>()
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

#[derive(Clone)]
pub struct ConcreteMappedStates<T: States> {
  pub original: T,
  pub mapping: Vec<usize>,
  pub nexts: Continuation
}

pub struct MinimizedState<'s, T: States> {
  states: &'s ConcreteMappedStates<T>,
  state: usize
}

impl<T: States> States for ConcreteMappedStates<T> {
  type State<'a> = MinimizedState<'a, T> where T: 'a;
  fn get_index(&self, state: &Self::State<'_>) -> Option<usize> {
    Some(state.state)
  }
  fn get_state(&self, index: usize) -> Option<Self::State<'_>> {
    Some(MinimizedState { states: self, state: index })
  }
}

impl<T: States> HasLength for ConcreteMappedStates<T> {
  fn len(&self) -> usize {
    self.nexts.len()
  }
}

impl<'a, T: States> StateProxy for MinimizedState<'a, T> {
  type Branch = usize;
  type BranchIter = NumIter;
  type SelfIter = NextIter<'a, T>;
  fn next_pieces(&self) -> Self::BranchIter {
    NumIter { i: 0, total: self.states.nexts.cont_index[self.state].len() }
  }
  fn next_states(&self, piece: Self::Branch) -> Self::SelfIter {
    let range = self.states.nexts.cont_index[self.state][piece];
    NextIter { states: self.states, range, pos: range.0 }
  }
}

pub struct NumIter {
  total: usize,
  i: usize
}

impl Iterator for NumIter {
  type Item = usize;
  fn next(&mut self) -> Option<Self::Item> {
    if self.i < self.total {
      let i = self.i;
      self.i += 1;
      Some(i)
    } else {
      None
    }
  }
}

pub struct NextIter<'a, T: States> {
  states: &'a ConcreteMappedStates<T>,
  range: (usize, usize),
  pos: usize
}

impl<'a, T: States> Iterator for NextIter<'a, T> {
  type Item = MinimizedState<'a, T>;
  fn next(&mut self) -> Option<Self::Item> {
    if self.pos >= self.range.1 {
      return None;
    }
    let result =
      Self::Item { states: self.states, state: self.states.nexts.continuations[self.pos] };
    self.pos += 1;
    Some(result)
  }
}

trait GetNext {
  fn get_next<'a, U: Into<Option<&'a [usize]>>+Copy>(
    &self,
    i: usize,
    res: U
  ) -> ArrayVec<Vec<usize>, 7>;
  fn get_next_id<'a, U: Into<Option<&'a [usize]>>+Copy>(&self, i: usize, res: U) -> Vec<usize>;
}

impl<T: States> GetNext for T {
  fn get_next<'a, U: Into<Option<&'a [usize]>>+Copy>(
    &self,
    i: usize,
    res: U
  ) -> ArrayVec<Vec<usize>, 7> {
    let state = self.get_state(i).unwrap();
    let mut nexts: ArrayVec<_, 7> = state
      .next_pieces()
      .into_iter()
      .map(|piece| {
        let mut next = state
          .next_states(piece)
          .map(|state| {
            let i = self.get_index(&state).unwrap();
            match res.into() {
              Some(res) => res[i],
              None => i
            }
          })
          .collect::<Vec<_>>();
        next.sort_unstable();
        next.dedup();
        next.shrink_to_fit();
        next
      })
      .collect();
    nexts.sort_unstable();
    let gcd = nexts.iter().count_same().fold(0, |a, (_v, b)| a.gcd(b));
    if gcd > 1 {
      nexts.into_iter().step_by(gcd).collect()
    } else {
      nexts
    }
  }
  fn get_next_id<'a, U: Into<Option<&'a [usize]>>+Copy>(&self, i: usize, res: U) -> Vec<usize> {
    let nexts = self.get_next(i, res);
    let mut ret = vec![nexts.len()];
    ret.extend(nexts.iter().map(|v| v.len()));
    ret.extend(nexts.iter().flatten());
    ret
  }
}

pub struct CountSame<Item: PartialEq, I: IntoIterator<Item=Item>> {
  iter: I::IntoIter,
  last: Option<I::Item>,
  count: usize
}

impl<Item: PartialEq, I: IntoIterator<Item=Item>> Iterator for CountSame<Item, I> {
  type Item = (I::Item, usize);
  fn next(&mut self) -> Option<Self::Item> {
    for item in self.iter.by_ref() {
      if self.last.is_none() {
        self.last = Some(item);
        self.count = 1;
      } else {
        let last = self.last.take().unwrap();
        if last == item {
          self.last = Some(item);
          self.count += 1;
        } else {
          self.last = Some(item);
          let count = self.count;
          self.count = 1;
          return Some((last, count));
        }
      }
    }
    if self.last.is_some() {
      let last = self.last.take().unwrap();
      self.last = None;
      Some((last, self.count))
    } else {
      None
    }
  }
}

pub trait CountSameExt<Item: PartialEq, I: IntoIterator<Item=Item>> {
  fn count_same(self) -> CountSame<Item, I>;
}

impl<Item: PartialEq, I: IntoIterator<Item=Item>> CountSameExt<Item, I> for I {
  fn count_same(self) -> CountSame<Item, I> {
    CountSame { iter: self.into_iter(), last: None, count: 0 }
  }
}
