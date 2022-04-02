use c4w::basics::{Field, Piece, PIECES};
use std::collections::{HashMap, VecDeque};

#[derive(Copy, Clone)]
struct RotatedPiece([u8; 4]);

impl RotatedPiece {
  #[allow(dead_code)]
  fn move_left(&self) -> Option<Self> {
    if self.0[0] != 0 {
      None
    } else {
      Some(RotatedPiece([self.0[1], self.0[2], self.0[3], 0]))
    }
  }
  fn move_right(&self) -> Option<Self> {
    if self.0[3] != 0 {
      None
    } else {
      Some(RotatedPiece([0, self.0[0], self.0[1], self.0[2]]))
    }
  }
  fn move_down(&self) -> Option<Self> {
    if self.0.iter().any(|&x| x & 1 != 0) {
      None
    } else {
      Some(RotatedPiece([self.0[0] >> 1, self.0[1] >> 1, self.0[2] >> 1, self.0[3] >> 1]))
    }
  }
}

#[derive(Copy, Clone)]
enum LineClear {
  Harddrop(Field),
  Softdrop(Field, RotatedPiece, Field),
}

fn get_piece(p: &Piece) -> Vec<RotatedPiece> {
  let mut result = Vec::new();
  let mut push_until_right_most = |shape: [u8; 4]| {
    let mut piece = RotatedPiece(shape);
    loop {
      result.push(piece);
      let p = piece.move_right();
      if let Some(p) = p {
        piece = p;
      } else {
        break;
      }
    }
  };
  match p {
    Piece::I => {
      push_until_right_most([0b10000000, 0b10000000, 0b10000000, 0b10000000]);
      push_until_right_most([0b11110000, 0b00000000, 0b00000000, 0b00000000]);
    }
    Piece::O => {
      push_until_right_most([0b11000000, 0b11000000, 0b00000000, 0b00000000]);
    }
    Piece::T => {
      push_until_right_most([0b10000000, 0b11000000, 0b10000000, 0b00000000]);
      push_until_right_most([0b11100000, 0b01000000, 0b00000000, 0b00000000]);
      push_until_right_most([0b01000000, 0b11000000, 0b01000000, 0b00000000]);
      push_until_right_most([0b01000000, 0b11100000, 0b00000000, 0b00000000]);
    }
    Piece::S => {
      push_until_right_most([0b01000000, 0b11000000, 0b10000000, 0b00000000]);
      push_until_right_most([0b11000000, 0b01100000, 0b00000000, 0b00000000]);
    }
    Piece::Z => {
      push_until_right_most([0b10000000, 0b11000000, 0b01000000, 0b00000000]);
      push_until_right_most([0b01100000, 0b11000000, 0b00000000, 0b00000000]);
    }
    Piece::J => {
      push_until_right_most([0b11000000, 0b01000000, 0b01000000, 0b00000000]);
      push_until_right_most([0b11100000, 0b10000000, 0b00000000, 0b00000000]);
      push_until_right_most([0b10000000, 0b10000000, 0b11000000, 0b00000000]);
      push_until_right_most([0b00100000, 0b11100000, 0b00000000, 0b00000000]);
    }
    Piece::L => {
      push_until_right_most([0b01000000, 0b01000000, 0b11000000, 0b00000000]);
      push_until_right_most([0b11100000, 0b00100000, 0b00000000, 0b00000000]);
      push_until_right_most([0b11000000, 0b10000000, 0b10000000, 0b00000000]);
      push_until_right_most([0b10000000, 0b11100000, 0b00000000, 0b00000000]);
    }
  }
  result
}

trait FieldDummy {
  fn flip_vertically(&self) -> Self;
  fn clearable(&self) -> bool;
  fn clear_line(&self) -> (usize, Self);
  fn overlap(&self, piece: &RotatedPiece) -> bool;
  fn put(&self, piece: &RotatedPiece) -> Self;
  fn possible_positions(&self, piece: &RotatedPiece) -> Vec<LineClear>;
}

impl FieldDummy for Field {
  fn flip_vertically(&self) -> Self {
    Field([self.0[3], self.0[2], self.0[1], self.0[0]])
  }
  fn clearable(&self) -> bool {
    let culmulated = self.0.iter().fold(!0 as u8, |acc, &x| acc & x);
    culmulated != 0
  }
  fn clear_line(&self) -> (usize, Self) {
    let mut line_count = 0;
    let mut field = *self;
    if !field.clearable() {
      return (0, field);
    }
    let culmulated = field.0.iter().fold(!0 as u8, |acc, &x| acc & x);
    for i in 0..8 {
      if culmulated & (1 << i) == 0 {
        continue;
      }
      let mask: u8 = (!0) << (i - line_count);
      for j in 0..4 {
        field.0[j] = field.0[j] & !mask | (field.0[j] >> 1) & mask;
      }
      line_count += 1;
    }
    (line_count, field)
  }
  fn overlap(&self, piece: &RotatedPiece) -> bool {
    for i in 0..4 {
      if self.0[i] & piece.0[i] != 0 {
        return true;
      }
    }
    false
  }
  fn put(&self, piece: &RotatedPiece) -> Self {
    let mut field = *self;
    for i in 0..4 {
      field.0[i] |= piece.0[i];
    }
    field
  }
  fn possible_positions(&self, piece: &RotatedPiece) -> Vec<LineClear> {
    let mut piece = *piece;
    let mut result_fields = Vec::new();
    let mut last_push = false;
    let mut harddrop = true;
    loop {
      let mut this_push = false;
      if !self.overlap(&piece) {
        if last_push {
          result_fields.pop();
        }
        let field = self.put(&piece);
        if field.clearable() {
          let (_, field) = field.clear_line();
          result_fields.push(if harddrop {
            LineClear::Harddrop(field)
          } else {
            LineClear::Softdrop(*self, piece, field)
          });
          this_push = true;
        }
      } else {
        harddrop = false;
      }
      last_push = this_push;
      let p = piece.move_down();
      if let Some(p) = p {
        piece = p;
      } else {
        break;
      }
    }
    result_fields
  }
}

fn main() {
  let mut continuation = HashMap::new();
  let mut queue = VecDeque::new();
  queue.push_back(Field([0b00000111, 0b00000000, 0b00000000, 0b00001111]));
  while !queue.is_empty() {
    let field = queue.pop_front().unwrap();
    if continuation.contains_key(&field) {
      continue;
    }
    let mut nexts = HashMap::new();
    for piece in PIECES.iter() {
      let mut v = vec![];
      for rotated_piece in get_piece(piece) {
        for position in field.possible_positions(&rotated_piece) {
          if let Some(new_field) = match position {
            LineClear::Harddrop(new_field) => Some(new_field),
            LineClear::Softdrop(old_field, piece, new_field) => {
              let mut buffer = String::new();
              print(&old_field, Some(&piece));
              std::io::stdin().read_line(&mut buffer).unwrap();
              if buffer.trim() == "y" || buffer.trim() == "Y" {
                Some(new_field)
              } else {
                None
              }
            }
          } {
            v.push(new_field);
            queue.push_back(new_field);
          }
        }
      }
      nexts.insert(piece, v);
    }
    continuation.insert(field, nexts);
  }
  bincode::serialize_into(std::io::stdout(), &continuation).unwrap();
}

fn print(field: &Field, piece: Option<&RotatedPiece>) {
  let mut result = [["  "; 4]; 8];
  for i in 0..4 {
    for (j, item) in result.iter_mut().enumerate() {
      if field.0[i] & (1 << j) != 0 {
        item[i] = "XX";
      }
    }
  }
  if let Some(piece) = piece {
    for i in 0..4 {
      for (j, item) in result.iter_mut().enumerate() {
        if piece.0[i] & (1 << j) != 0 {
          if item[i] == "  " {
            item[i] = "[]";
          } else {
            item[i] = "**";
          }
        }
      }
    }
  }
  for i in (0..8).rev() {
    eprint!("{} |", i);
    for j in 0..4 {
      eprint!("{}", result[i][j]);
    }
    eprintln!("|");
  }
  eprintln!("  +--------+");
}