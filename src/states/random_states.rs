use super::*;
use crate::basics::{Field, Piece, PIECES};
use std::collections::HashMap;

pub struct RandomStates {
  fields: Vec<Field>,
  continuations: Continuation,
  preview: usize,
  hold: bool
}

impl States for RandomStates {
  type State = RandomState;
  type Branch = (Self::State, Piece);
  fn decode(&self, index: usize) -> Option<Self::State> {
    let (seq, field) = index.div_rem(&self.fields.len());
    Some(RandomState { field, seq: seq as u64 })
  }
  fn encode(&self, state: &Self::State) -> Option<usize> {
    Some(self.fields.len() * state.seq as usize + state.field)
  }
  fn next_pieces(&self, state: Self::State) -> impl Iterator<Item=Self::Branch> {
    PIECES.iter().cloned().map(move |i| (state, i))
  }
  fn next_states(&self, piece: Self::Branch) -> impl Iterator<Item=Self::State> {
    let (state, piece) = piece;
    let length = if self.hold { self.preview + 1 } else { self.preview };
    let (seq, current) = state.seq.clone().push(piece, length);
    let (begin, end) = self.continuations.cont_index[state.field][current as usize];
    if self.hold {
      let (seq2, current) = seq.clone().swap(current);
      let (begin2, end2) = self.continuations.cont_index[state.field][current as usize];
      RandomStateIter {
        states: self,
        seq,
        seq2: Some(seq2),
        range: (begin, end),
        range2: Some((begin2, end2)),
        pos: begin
      }
    } else {
      RandomStateIter {
        states: self,
        seq,
        seq2: None,
        range: (begin, end),
        range2: None,
        pos: begin
      }
    }
  }
}

impl<'a> Creatable<'a> for RandomStates {
  fn new(
    continuations: &'a HashMap<Field, HashMap<Piece, Vec<Field>>>,
    preview: usize,
    hold: bool
  ) -> Self {
    let (fields, continuations) = Continuation::new(continuations);
    assert!(
      (fields.len() as f64).log2()
        + (PIECES.len() as f64).log2() * (preview as f64 + if hold { 1.0 } else { 0.0 })
        <= u64::BITS as f64
    );
    RandomStates { fields, continuations, preview, hold }
  }
}

impl HasLength for RandomStates {
  fn len(&self) -> usize {
    self.fields.len()
      * PIECES.len().pow(self.preview as u32)
      * (if self.hold { PIECES.len() } else { 1 })
  }
}

#[derive(Clone, Copy)]
pub struct RandomState {
  field: usize,
  seq: u64
}

pub struct RandomStateIter<'a> {
  states: &'a RandomStates,
  seq: u64,
  seq2: Option<u64>,
  range: (usize, usize),
  range2: Option<(usize, usize)>,
  pos: usize
}

impl<'a> Iterator for RandomStateIter<'a> {
  type Item = RandomState;
  fn next(&mut self) -> Option<Self::Item> {
    if self.pos >= self.range.1 {
      if let Some((begin, end)) = self.range2.take() {
        self.seq = self.seq2.take().unwrap();
        self.range = (begin, end);
        self.pos = begin;
        return self.next();
      } else {
        return None;
      }
    }
    let result = RandomState {
      field: self.states.continuations.continuations[self.pos],
      seq: self.seq.clone()
    };
    self.pos += 1;
    Some(result)
  }
}

#[derive(Ord, PartialEq, PartialOrd, Eq, Clone)]
pub struct FieldWithPiece(Field, Option<Piece>);

impl std::fmt::Display for FieldWithPiece {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    if let Some(piece) = self.1 {
      write!(f, "{}\nHold: {:?}\n", self.0, piece)
    } else {
      write!(f, "{}\nHold: None\n", self.0)
    }
  }
}
