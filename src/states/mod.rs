mod random_states;
pub use random_states::*;
use arrayvec::ArrayVec;

use std::collections::HashMap;
use crate::basics::{Field, Piece, PIECES};

pub trait StateProxy {
  type Branch;
  type BranchIter: Iterator<Item=Self::Branch>;
  type SelfIter: Iterator<Item=Self>;
  fn next_pieces(&self) -> Self::BranchIter;
  fn next_states(&self, piece: Self::Branch) -> Self::SelfIter;
}

pub trait PrintableStateProxy: StateProxy {
  type MarkovState: std::fmt::Display + Ord + PartialEq + Clone + Send;
  fn markov_state(&self) -> Option<Self::MarkovState>;
}

pub trait Creatable<'a> {
  fn new(continuations: &'a HashMap<Field, HashMap<Piece, Vec<Field>>>, preview: usize, hold: bool) -> Self;
}

pub trait HasLength {
  fn len(&self) -> usize;
  fn is_empty(&self) -> bool {
    self.len() == 0
  }
}

pub trait States {
  type State: StateProxy;
  fn get_state(&self, index: usize) -> Option<Self::State>;
  fn get_index(&self, state: &Self::State) -> Option<usize>;
}

pub struct Continuation {
  pub cont_index: Vec<ArrayVec<(usize, usize), 7>>,
  pub continuations: Vec<usize>,
}

impl Continuation {
  fn new(continuations: &HashMap<Field, HashMap<Piece, Vec<Field>>>) -> (Vec<Field>, Self) {
    let fields = continuations.keys().cloned().collect::<Vec<Field>>();
    let field2num = fields.iter().enumerate().map(|(i, f)| (*f, i)).collect::<HashMap<_, _>>();
    let mut cont_index: Vec<ArrayVec<(usize, usize), 7>> = Vec::new();
    let mut cont = Vec::new();
    for &field in &fields {
      cont_index.push((0..PIECES.len()).map(|i| {
        let begin = cont.len();
        let piece = Piece::num2piece(i);
        for &next_field in &continuations[&field][&piece] {
          cont.push(field2num[&next_field]);
        }
        (begin, cont.len())
      }).collect());
    }
    (fields, Continuation {
      cont_index,
      continuations: cont,
    })
  }
  pub fn len(&self) -> usize {
    self.cont_index.len()
  }
  pub fn is_empty(&self) -> bool {
    self.len() == 0
  }
}

impl<S: IntoIterator<Item=usize>, T: IntoIterator<Item=S>> FromIterator<T> for Continuation {
  fn from_iter<I: IntoIterator<Item=T>>(iter: I) -> Self {
    let mut cont = Continuation {cont_index: vec![], continuations: vec![]};
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