use crate::states::*;
mod dashmap;
pub use self::dashmap::*;
mod recorder;
pub use recorder::*;


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
    NumIter { i: 0, I: self.states.nexts.cont_index[self.state].len() }
  }
  fn next_states(self: &Self, piece: Self::Branch) -> Self::SelfIter {
    let range = self.states.nexts.cont_index[self.state][piece];
    NextIter {
      states: self.states,
      range,
      pos: range.0,
    }
  }
}

pub struct NumIter {
  I: usize,
  i: usize
}

impl Iterator for NumIter {
  type Item = usize;
  fn next(&mut self) -> Option<Self::Item> {
    if self.i < self.I {
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
