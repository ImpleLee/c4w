mod mapped_states;
pub use mapped_states::*;
mod concrete_mapped_states;
pub use concrete_mapped_states::*;
mod field_sequence_states;
pub use field_sequence_states::*;

use crate::basics::{Field, Piece, PIECES};
use arrayvec::ArrayVec;
use itertools::Itertools;
use num_integer::Integer;
use std::collections::{HashMap, VecDeque};

/* pub trait PrintableStateProxy: StateProxy {
  type MarkovState: std::fmt::Display+Ord+PartialEq+Clone+Send;
  fn markov_state(&self) -> Option<Self::MarkovState>;
} */

pub trait Creatable<'a> {
  fn new(
    continuations: &'a HashMap<Field, HashMap<Piece, Vec<Field>>>,
    preview: usize,
    hold: bool
  ) -> Self;
}

pub trait HasLength {
  fn len(&self) -> usize;
  fn is_empty(&self) -> bool {
    self.len() == 0
  }
}

pub trait States: HasLength+std::marker::Sync {
  type State;
  type Branch;
  fn decode(&self, index: usize) -> Option<Self::State>;
  fn encode(&self, state: &Self::State) -> Option<usize>;
  fn next_pieces(&self, state: Self::State) -> impl Iterator<Item=Self::Branch>;
  fn next_states(&self, piece: Self::Branch) -> impl Iterator<Item=Self::State>;
}

#[derive(Clone)]
pub struct Continuation<T=usize> {
  pub cont_index: Vec<ArrayVec<(usize, usize), 7>>,
  pub continuations: Vec<T>
}

impl Continuation<usize> {
  fn new(continuations: &HashMap<Field, HashMap<Piece, Vec<Field>>>) -> (Vec<Field>, Self) {
    let fields = continuations.keys().cloned().collect::<Vec<Field>>();
    let field2num = fields.iter().enumerate().map(|(i, f)| (*f, i)).collect::<HashMap<_, _>>();
    let mut cont_index: Vec<ArrayVec<(usize, usize), 7>> = Vec::new();
    let mut cont = Vec::new();
    for &field in &fields {
      cont_index.push(
        (0..PIECES.len())
          .map(|i| {
            let begin = cont.len();
            let piece = Piece::num2piece(i);
            for &next_field in &continuations[&field][&piece] {
              cont.push(field2num[&next_field]);
            }
            (begin, cont.len())
          })
          .collect()
      );
    }
    (fields, Continuation { cont_index, continuations: cont })
  }
}

impl<T> HasLength for Continuation<T> {
  fn len(&self) -> usize {
    self.cont_index.len()
  }
}

impl<A, S: IntoIterator<Item=A>, T: IntoIterator<Item=S>> FromIterator<T> for Continuation<A> {
  fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
    let mut cont = Continuation { cont_index: vec![], continuations: vec![] };
    for next in iter {
      let mut cont_index = ArrayVec::new();
      for next in next {
        let begin = cont.continuations.len();
        cont.continuations.extend(next);
        let end = cont.continuations.len();
        cont_index.push((begin, end));
      }
      cont.cont_index.push(cont_index);
    }
    cont
  }
}

pub trait Sequence: Sized+Clone {
  // if there is hold, it is pushed out
  // the most recent piece becomes the hold
  // (swap semantics)
  fn push(self, piece: Piece, length: usize) -> (Self, Piece);
  // actual swap
  // exchange the hold and the piece
  // should only be called if there is a hold
  fn swap(self, piece: Piece) -> (Self, Piece);
}

impl Sequence for u64 {
  fn push(self, piece: Piece, length: usize) -> (Self, Piece) {
    let seq = self + piece as u64 * (PIECES.len() as u64).pow(length as u32);
    let (seq, current) = seq.div_rem(&(PIECES.len() as u64));
    (seq, Piece::num2piece(current as usize))
  }
  fn swap(self, piece: Piece) -> (Self, Piece) {
    let swapped = self % (PIECES.len() as u64);
    (self - swapped + piece as u64, Piece::num2piece(swapped as usize))
  }
}

impl<'a> Sequence for VecDeque<Piece> {
  fn push(mut self, piece: Piece, _: usize) -> (Self, Piece) {
    self.push_back(piece);
    let current = self.pop_front().unwrap();
    (self, current)
  }
  fn swap(mut self, piece: Piece) -> (Self, Piece) {
    let current = self.pop_front().unwrap();
    self.push_front(piece);
    (self, current)
  }
}

pub trait GetNext {
  fn true_get_next<'a, I: Ord, F: Fn(Vec<usize>) -> I>(
    &self,
    i: usize,
    maximal_func: F
  ) -> ArrayVec<I, 7>;
  fn get_next<'a, U: Into<Option<&'a [usize]>>+Copy>(
    &self,
    i: usize,
    res: U
  ) -> ArrayVec<Vec<usize>, 7> {
    if let Some(res) = res.into() {
      self.true_get_next(i, |v| {
        let mut v2 = v.into_iter().map(|i| res[i]).collect::<Vec<_>>();
        v2.sort_unstable();
        v2.dedup();
        v2
      })
    } else {
      self.true_get_next(i, |mut v| {
        v.sort_unstable();
        v.dedup();
        v
      })
    }
  }
  fn get_next_id<'a, U: Into<Option<&'a [usize]>>+Copy>(&self, i: usize, res: U) -> Vec<usize> {
    next2id(self.get_next(i, res))
  }
}

fn next2id(nexts: ArrayVec<Vec<usize>, 7>) -> Vec<usize> {
  std::iter::once(nexts.len())
    .chain(nexts.iter().map(|v| v.len()))
    .chain(nexts.iter().flatten().cloned())
    .collect()
}

impl<T: States> GetNext for T {
  fn true_get_next<'a, I: Ord, F: Fn(Vec<usize>) -> I>(
    &self,
    i: usize,
    maximal_func: F
  ) -> ArrayVec<I, 7> {
    let state = self.decode(i).unwrap();
    let mut nexts: ArrayVec<_, 7> = self
      .next_pieces(state)
      .into_iter()
      .map(|piece| {
        let next =
          self.next_states(piece).map(|state| self.encode(&state).unwrap()).collect_vec();
        maximal_func(next)
      })
      .collect();
    nexts.sort_unstable();
    let gcd = nexts.iter().count_same().fold(0, |a, (_, b)| a.gcd(&b));
    if gcd > 1 {
      nexts.into_iter().step_by(gcd).collect()
    } else {
      nexts
    }
  }
}

struct CountSame<Item: PartialEq, I: IntoIterator<Item=Item>> {
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

trait CountSameExt<Item: PartialEq, I: IntoIterator<Item=Item>> {
  fn count_same(self) -> CountSame<Item, I>;
}

impl<Item: PartialEq, I: IntoIterator<Item=Item>> CountSameExt<Item, I> for I {
  fn count_same(self) -> CountSame<Item, I> {
    CountSame { iter: self.into_iter(), last: None, count: 0 }
  }
}
