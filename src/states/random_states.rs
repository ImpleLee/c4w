use super::*;
use crate::basics::{Field, Piece, PIECES};
use std::collections::HashMap;

pub struct RandomStates {
  fields: Vec<Field>,
  continuations: Continuation,
  preview: usize,
  hold: bool
}

impl<'a> States for &'a RandomStates {
  type State = RandomState<'a, u64>;
  fn get_state(&self, index: usize) -> Option<Self::State> {
    let (seq, field) = index.div_rem(&self.fields.len());
    Some(RandomState { states: self, field, seq: seq as u64 })
  }
  fn get_index(&self, state: &Self::State) -> Option<usize> {
    Some(self.fields.len() * state.seq as usize + state.field)
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

impl HasLength for &RandomStates {
  fn len(&self) -> usize {
    self.fields.len()
      * PIECES.len().pow(self.preview as u32)
      * (if self.hold { PIECES.len() } else { 1 })
  }
}

pub struct RandomState<'s, T: Sequence> {
  states: &'s RandomStates,
  field: usize,
  seq: T
}

impl<'s, T: Sequence> StateProxy for RandomState<'s, T> {
  type Branch = Piece;
  type BranchIter = PieceIter;
  type SelfIter = RandomStateIter<'s, T>;
  fn next_pieces(&self) -> Self::BranchIter {
    PieceIter { piece: 0 }
  }
  fn next_states(&self, piece: Self::Branch) -> Self::SelfIter {
    let length = if self.states.hold { self.states.preview + 1 } else { self.states.preview };
    let (seq, current) = self.seq.clone().push(piece, length);
    let (begin, end) = self.states.continuations.cont_index[self.field][current as usize];
    if self.states.hold {
      let (seq2, current) = seq.clone().swap(current);
      let (begin2, end2) = self.states.continuations.cont_index[self.field][current as usize];
      RandomStateIter {
        states: self.states,
        seq,
        seq2: Some(seq2),
        range: (begin, end),
        range2: Some((begin2, end2)),
        pos: begin
      }
    } else {
      RandomStateIter {
        states: self.states,
        seq,
        seq2: None,
        range: (begin, end),
        range2: None,
        pos: begin
      }
    }
  }
}

pub struct RandomStateIter<'a, T: Sequence> {
  states: &'a RandomStates,
  seq: T,
  seq2: Option<T>,
  range: (usize, usize),
  range2: Option<(usize, usize)>,
  pos: usize
}

impl<'a, T: Sequence> Iterator for RandomStateIter<'a, T> {
  type Item = RandomState<'a, T>;
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
      states: self.states,
      field: self.states.continuations.continuations[self.pos],
      seq: self.seq.clone()
    };
    self.pos += 1;
    Some(result)
  }
}

pub struct PieceIter {
  piece: usize
}

impl Iterator for PieceIter {
  type Item = Piece;
  fn next(&mut self) -> Option<Self::Item> {
    if self.piece >= PIECES.len() {
      None
    } else {
      let result = Piece::num2piece(self.piece);
      self.piece += 1;
      Some(result)
    }
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
