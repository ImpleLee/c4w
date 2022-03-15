use std::collections::{HashMap, VecDeque};
use crate::basics::{Field, Piece, PIECES};
use super::*;
use num_integer::Integer;

pub struct RandomStates<'a> {
  continuations: &'a HashMap<Field, HashMap<Piece, Vec<Field>>>,
  fields: Vec<Field>,
  field2num: HashMap<Field, usize>,
  preview: usize,
  hold: bool,
}

impl<'s, 'l: 's> States for &'s RandomStates<'l> {
  type State = RandomState<'s, 'l>;
  
  fn get_state(&self, index: usize) -> Option<Self::State> {
    if index >= self.len() {
      return None;
    }
    let (hold, mut index) = if self.hold {
      let (index, hold) = index.div_rem(&PIECES.len());
      (Some(Piece::num2piece(hold)), index)
    } else {
      (None, index)
    };
    let mut pieces = VecDeque::new();
    for _ in 0..self.preview {
      let (field_index, piece_index) = index.div_rem(&PIECES.len());
      pieces.push_front(Piece::num2piece(piece_index));
      index = field_index;
    }
    Some(Self::State {
      states: &self,
      field: self.fields[index],
      pieces,
      hold,
    })
  }
  
  fn get_index(&self, state: &Self::State) -> Option<usize> {
    let mut index = self.field2num[&state.field];
    for &piece in &state.pieces {
      index *= PIECES.len();
      index += piece as usize;
    }
    if let Some(hold) = state.hold {
      index *= PIECES.len();
      index += hold as usize;
    }
    Some(index)
  }
}

impl<'a> Creatable<'a> for RandomStates<'a> {
  fn new(continuations: &'a HashMap<Field, HashMap<Piece, Vec<Field>>>, preview: usize, hold: bool) -> Self {
    let fields = continuations.keys().cloned().collect::<Vec<Field>>();
    let field2num = fields.iter().enumerate().map(|(i, f)| (f.clone(), i)).collect();
    RandomStates {
      continuations,
      fields,
      field2num,
      preview,
      hold,
    }
  }
}

impl HasLength for &RandomStates<'_> {
  fn len(&self) -> usize {
    self.fields.len() * PIECES.len().pow(self.preview as u32) * (if self.hold { PIECES.len() } else { 1 })
  }
}

pub struct RandomState<'s, 'l: 's> {
  states: &'s RandomStates<'l>,
  field: Field,
  pieces: VecDeque<Piece>,
  hold: Option<Piece>,
}

impl<'s, 'l: 's> StateProxy for RandomState<'s, 'l> {
  type Branch = Piece;
  type MarkovState = FieldWithPiece;
  type BranchIter = std::vec::IntoIter<Self::Branch>;
  type SelfIter = std::vec::IntoIter<Self>;
  fn next_pieces(self: &Self) -> Self::BranchIter {
    PIECES.iter().cloned().collect::<Vec<_>>().into_iter()
  }
  fn next_states(self: &Self, piece: Self::Branch) -> Self::SelfIter {
    let mut result = Vec::new();
    let mut pieces = self.pieces.clone();
    pieces.push_back(piece);
    let piece = pieces.pop_front().unwrap();
    let mut push = |piece: Piece, hold: Option<Piece>| {
      for &field in self.states.continuations[&self.field][&piece].iter() {
        result.push(RandomState {
          states: self.states,
          field,
          pieces: pieces.clone(),
          hold,
        });
      }
    };
    push(piece, self.hold);
    if let Some(hold) = self.hold {
      push(hold, Some(piece));
    }
    result.into_iter()
  }
  fn markov_state(self: &Self) -> Option<Self::MarkovState> {
    Some(FieldWithPiece(self.field, self.hold))
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
