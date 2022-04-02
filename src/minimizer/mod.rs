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
      nexts.push(next);
    }
    if nexts.len() > 1 && nexts[1..].iter().all(|x| x == &nexts[0]) {
      let t = nexts[0].clone();
      nexts.clear();
      nexts.push(t);
    } else {
      nexts.sort();
    }
    nexts
  }
}
