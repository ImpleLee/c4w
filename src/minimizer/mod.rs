use crate::states::*;
mod dashmap;
pub use self::dashmap::*;
mod recorder;
pub use recorder::*;
mod raw;
pub use raw::*;
mod parallel;
pub use parallel::*;
use arrayvec::ArrayVec;
use gcd::Gcd;

pub trait Minimizer<T: States> {
  fn minimize(states: T) -> MinimizedStates<T>;
}

pub struct MinimizedStates<T: States> {
  states: T,
  state2num: Vec<usize>,
  pub nexts: Continuation,
}

pub struct MinimizedState<'s, T: States> {
  states: &'s MinimizedStates<T>,
  state: usize,
}

impl<'a, T: States> States for &'a MinimizedStates<T> {
  type State = MinimizedState<'a, T>;
  fn get_index(&self, state: &Self::State) -> Option<usize> {
    Some(state.state)
  }
  fn get_state(&self, index: usize) -> Option<Self::State> {
    Some(MinimizedState {
      states: self,
      state: index,
    })
  }
}

impl<T: States> HasLength for &MinimizedStates<T> {
  fn len(&self) -> usize {
    self.nexts.len()
  }
}

impl<'a, T: States> StateProxy for MinimizedState<'a, T> {
  type Branch = usize;
  type BranchIter = NumIter;
  type SelfIter = NextIter<'a, T>;
  fn next_pieces(self: &Self) -> Self::BranchIter {
    NumIter { i: 0, total: self.states.nexts.cont_index[self.state].len() }
  }
  fn next_states(&self, piece: Self::Branch) -> Self::SelfIter {
    let range = self.states.nexts.cont_index[self.state][piece];
    NextIter {
      states: self.states,
      range,
      pos: range.0,
    }
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
  states: &'a MinimizedStates<T>,
  range: (usize, usize),
  pos: usize,
}

impl<'a, T: States> Iterator for NextIter<'a, T> {
  type Item = MinimizedState<'a, T>;
  fn next(&mut self) -> Option<Self::Item> {
    if self.pos >= self.range.1 {
      return None;
    }
    let result = Self::Item {
      states: self.states,
      state: self.states.nexts.continuations[self.pos],
    };
    self.pos += 1;
    Some(result)
  }
}

trait GetNext {
  fn get_next(&self, i: usize, res: &Vec<usize>) -> ArrayVec<Vec<usize>, 7>;
}

impl<T: States> GetNext for T {
  fn get_next(&self, i: usize, res: &Vec<usize>) -> ArrayVec<Vec<usize>, 7> {
    let state = self.get_state(i).unwrap();
    let mut nexts = ArrayVec::new();
    for piece in state.next_pieces() {
      let mut next = Vec::new();
      for state in state.next_states(piece) {
        next.push(res[self.get_index(&state).unwrap()]);
      }
      next.sort_unstable();
      next.dedup();
      next.shrink_to_fit();
      nexts.push(next);
    }
    nexts.sort_unstable();
    let gcd = nexts.iter()
      .count_same()
      .fold(0, |a, (_v, b)| a.gcd(b));
    if gcd > 1 {
      nexts.into_iter().step_by(gcd).collect()
    } else {
      nexts
    }
  }
}

struct CountSame<I: IntoIterator> where I::Item: PartialEq {
  iter: I::IntoIter,
  last: Option<I::Item>,
  count: usize,
}

impl<I: IntoIterator> Iterator for CountSame<I> where I::Item: PartialEq {
  type Item = (I::Item, usize);
  fn next(&mut self) -> Option<Self::Item> {
    while let Some(item) = self.iter.next() {
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

trait CountSameExt<I: IntoIterator> where I::Item: PartialEq {
  fn count_same(self) -> CountSame<I>;
}

impl<I: IntoIterator> CountSameExt<I> for I where I::Item: PartialEq {
  fn count_same(self) -> CountSame<I> {
    CountSame {
      iter: self.into_iter(),
      last: None,
      count: 0,
    }
  }
}