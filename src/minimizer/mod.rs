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
  fn minimize<'a, T: States<'a>+std::marker::Sync+HasLength>(states: &'a T) -> MinimizedStates;
  fn minimize_again(mut minimized: MinimizedStates) -> MinimizedStates {
    let mut minimized_again = Self::minimize(&minimized);
    minimized.state2num.par_iter_mut().for_each(|i| *i = minimized_again.state2num[*i]);
    minimized_again.state2num = minimized.state2num;
    minimized_again
  }
}

#[derive(Clone)]
pub struct MinimizedStates {
  pub state2num: Vec<usize>,
  pub nexts: Continuation
}

pub struct MinimizedState<'s> {
  states: &'s MinimizedStates,
  state: usize
}

impl<'a> States<'a> for MinimizedStates {
  type State = MinimizedState<'a>;
  fn get_index(&'a self, state: &Self::State) -> Option<usize> {
    Some(state.state)
  }
  fn get_state(&'a self, index: usize) -> Option<Self::State> {
    Some(MinimizedState { states: self, state: index })
  }
}

impl HasLength for MinimizedStates {
  fn len(&self) -> usize {
    self.nexts.len()
  }
}

impl<'a> StateProxy for MinimizedState<'a> {
  type Branch = usize;
  type BranchIter = NumIter;
  type SelfIter = NextIter<'a>;
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

pub struct NextIter<'a> {
  states: &'a MinimizedStates,
  range: (usize, usize),
  pos: usize
}

impl<'a> Iterator for NextIter<'a> {
  type Item = MinimizedState<'a>;
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

trait GetNext<'b> {
  fn get_next<'a, U: Into<Option<&'a [usize]>>+Copy>(
    &'b self,
    i: usize,
    res: U
  ) -> ArrayVec<Vec<usize>, 7>;
  fn get_next_id<'a, U: Into<Option<&'a [usize]>>+Copy>(&'b self, i: usize, res: U) -> Vec<usize>;
}

impl<'b, T: States<'b>> GetNext<'b> for T {
  fn get_next<'a, U: Into<Option<&'a [usize]>>+Copy>(
    &'b self,
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
  fn get_next_id<'a, U: Into<Option<&'a [usize]>>+Copy>(&'b self, i: usize, res: U) -> Vec<usize> {
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
