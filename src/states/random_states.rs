use std::collections::HashMap;
use crate::basics::{Field, Piece, PIECES};
use super::*;
use num_integer::Integer;
use arrayvec::ArrayVec;

pub struct RandomStates {
  continuations: Continuation,
  preview: usize,
  hold: bool,
}

impl<'a> States for &'a RandomStates {
  type State = RandomState<'a>;
  fn get_state(self: &Self, index: usize) -> Option<Self::State> {
    let (seq, field) = index.div_rem(&self.continuations.fields.len());
    Some(RandomState {
      states: self,
      field,
      seq: seq as u64,
    })
  }
  fn get_index(self: &Self, state: &Self::State) -> Option<usize> {
    Some(self.continuations.fields.len() * state.seq as usize + state.field)
  }
}

impl<'a> Creatable<'a> for RandomStates {
  fn new(continuations: &'a HashMap<Field, HashMap<Piece, Vec<Field>>>, preview: usize, hold: bool) -> Self {
    RandomStates {
      continuations: Continuation::new(continuations),
      preview,
      hold,
    }
  }
}

impl HasLength for &RandomStates {
  fn len(&self) -> usize {
    self.continuations.fields.len() * PIECES.len().pow(self.preview as u32) * (if self.hold { PIECES.len() } else { 1 })
  }
}

pub struct RandomState<'s> {
  states: &'s RandomStates,
  field: usize,
  seq: u64,
}

impl<'s> StateProxy for RandomState<'s> {
  type Branch = Piece;
  type MarkovState = FieldWithPiece;
  type BranchIter = PieceIter;
  type SelfIter = RandomStateIter<'s>;
  fn next_pieces(self: &Self) -> Self::BranchIter {
    PieceIter{ piece: 0 }
  }
  fn next_states(self: &Self, piece: Self::Branch) -> Self::SelfIter {
    let length = if self.states.hold { self.states.preview + 1 } else { self.states.preview };
    let (seq, current) = self.seq.push(piece, length);
    let (begin, end) = self.states.continuations.cont_index[self.field][current as usize];
    if self.states.hold {
      let (seq2, current) = seq.swap(current);
      let (begin2, end2) = self.states.continuations.cont_index[self.field][current as usize];
      RandomStateIter {
        states: self.states,
        seq: seq,
        seq2: Some(seq2),
        range: (begin, end),
        range2: Some((begin2, end2)),
        pos: begin,
      }
    } else {
      RandomStateIter {
        states: self.states,
        seq: seq,
        seq2: None,
        range: (begin, end),
        range2: None,
        pos: begin,
      }
    }
  }
  fn markov_state(self: &Self) -> Option<Self::MarkovState> {
    let mut ret = FieldWithPiece(self.states.continuations.fields[self.field], None);
    if self.states.hold {
      ret.1 = Some(self.seq.swap(Piece::I).1)
    }
    Some(ret)
  }
}

pub struct RandomStateIter<'a> {
  states: &'a RandomStates,
  seq: u64,
  seq2: Option<u64>,
  range: (usize, usize),
  range2: Option<(usize, usize)>,
  pos: usize,
}

impl<'a> Iterator for RandomStateIter<'a> {
  type Item = RandomState<'a>;
  fn next(&mut self) -> Option<Self::Item> {
    if self.pos >= self.range.1 {
      if let Some((begin, end)) = self.range2.take() {
        self.seq = self.seq2.unwrap();
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
      seq: self.seq,
    };
    self.pos += 1;
    Some(result)
  }
}

pub struct PieceIter {
  piece: usize,
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

trait Sequence: Sized {
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

struct Continuation {
  fields: Vec<Field>,
  cont_index: Vec<ArrayVec<(usize, usize), 7>>,
  continuations: Vec<usize>,
}

impl Continuation {
  fn new(continuations: &HashMap<Field, HashMap<Piece, Vec<Field>>>) -> Self {
    let fields = continuations.keys().cloned().collect::<Vec<Field>>();
    let field2num = fields.iter().enumerate().map(|(i, f)| (f.clone(), i)).collect::<HashMap<_, _>>();
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
    Continuation {
      fields,
      cont_index,
      continuations: cont,
    }
  }
}