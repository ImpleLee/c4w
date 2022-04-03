use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Hash, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub struct Field(pub [u8; 4]);

impl std::fmt::Display for Field {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    let mut result = [["  "; 4]; 8];
    for i in 0..4 {
      for (j, item) in result.iter_mut().enumerate() {
        if self.0[i] & (1 << j) != 0 {
          item[i] = "XX";
        }
      }
    }
    write!(
      f,
      "{}",
      result
        .iter()
        .enumerate()
        .rev()
        .map(|(i, line)| {
          format!("{} |{}|", i, line.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(""))
        })
        .collect::<Vec<_>>()
        .join("\n")
    )
  }
}

#[derive(Debug, Hash, Eq, PartialEq, Serialize, Deserialize, Copy, Clone, Ord, PartialOrd)]
pub enum Piece {
  I = 0,
  O = 1,
  T = 2,
  S = 3,
  Z = 4,
  J = 5,
  L = 6
}

impl Piece {
  pub fn num2piece(num: usize) -> Piece {
    match num {
      0 => Piece::I,
      1 => Piece::O,
      2 => Piece::T,
      3 => Piece::S,
      4 => Piece::Z,
      5 => Piece::J,
      6 => Piece::L,
      _ => panic!("invalid piece number {}", num)
    }
  }
}

pub const PIECES: [Piece; 7] =
  [Piece::I, Piece::O, Piece::T, Piece::S, Piece::Z, Piece::J, Piece::L];
