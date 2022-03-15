use std::collections::{HashMap, VecDeque, HashSet};
use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;
use ordered_float::NotNan;
use itertools::Itertools;
use average::{Mean, Max, Estimate};
use rayon::prelude::*;

// four columns
#[derive(Copy, Clone, Hash, Eq, PartialEq, Debug, Serialize, Deserialize, Ord, PartialOrd)]
struct Field([u8; 4]);

// four columns
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
struct RotatedPiece([u8; 4]);

#[derive(Copy, Clone, Debug)]
enum LineClear {
  Harddrop(Field),
  Softdrop(Field, RotatedPiece, Field),
}

impl Field {
  fn flip_vertically(&self) -> Self {
    Field([self.0[3], self.0[2], self.0[1], self.0[0]])
  }
  fn clearable(&self) -> bool {
    let culmulated = self.0.iter().fold(!0 as u8, |acc, &x| acc & x);
    culmulated != 0
  }
  fn clear_line(&self) -> (usize, Self) {
    let mut line_count = 0;
    let mut field = self.clone();
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
    let mut field = self.clone();
    for i in 0..4 {
      field.0[i] |= piece.0[i];
    }
    field
  }
  fn possible_positions(&self, piece: &RotatedPiece) -> Vec<LineClear> {
    let mut piece = piece.clone();
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
            LineClear::Softdrop(self.clone(), piece.clone(), field)
          });
          this_push = true;
        }
      } else {
        harddrop = false;
      }
      last_push = this_push;
      let p = piece.move_down();
      if p.is_none() {
        break;
      } else {
        piece = p.unwrap();
      }
    }
    result_fields
  }
}

impl RotatedPiece {
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

#[derive(Debug, Hash, Eq, PartialEq, Serialize, Deserialize, Copy, Clone, Ord, PartialOrd)]
enum Piece {
  I = 0,
  O = 1,
  T = 2,
  S = 3,
  Z = 4,
  J = 5,
  L = 6,
}

impl Piece {
  fn get_piece(&self) -> Vec<RotatedPiece> {
    let mut result = Vec::new();
    let mut push_until_right_most = |shape: [u8; 4]| {
      let mut piece = RotatedPiece(shape);
      loop {
        result.push(piece);
        let p = piece.move_right();
        if p.is_none() {
          break;
        } else {
          piece = p.unwrap();
        }
      }
    };
    match self {
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

  fn num2piece(num: usize) -> Piece {
    match num {
      0 => Piece::I,
      1 => Piece::O,
      2 => Piece::T,
      3 => Piece::S,
      4 => Piece::Z,
      5 => Piece::J,
      6 => Piece::L,
      _ => panic!("invalid piece number"),
    }
  }
}

impl Default for Piece {
  fn default() -> Self {
    Piece::I
  }
}

const PIECES: [Piece; 7] = [
  Piece::I,
  Piece::O,
  Piece::T,
  Piece::S,
  Piece::Z,
  Piece::J,
  Piece::L,
];

fn print(field: &Field, piece: Option<&RotatedPiece>) {
  let mut result = [["  "; 4]; 8];
  for i in 0..4 {
    for j in 0..8 {
      if field.0[i] & (1 << j) != 0 {
        result[j][i] = "XX";
      }
    }
  }
  if let Some(piece) = piece {
    for i in 0..4 {
      for j in 0..8 {
        if piece.0[i] & (1 << j) != 0 {
          if result[j][i] == "  " {
            result[j][i] = "[]";
          } else {
            result[j][i] = "**";
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

trait State {
  fn new(fields: Vec<&Field>, preview: usize, hold: bool) -> Vec<Self> where Self: std::marker::Sized;

  fn next_pieces(self: &Self) -> Vec<Piece>;

  fn add_piece(self: &Self, piece: Piece) -> Vec<(Self, Piece)> where Self: std::marker::Sized;

  fn change_field(self: &Self, field: Field) -> Self;

  fn get_field(self: &Self) -> Field;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
struct RandomState {
  field: Field,
  preview: VecDeque<Piece>,
  hold: Option<Piece>
}

impl State for RandomState {
  fn new(fields: Vec<&Field>, preview: usize, hold: bool) -> Vec<Self> {
    let mut res = vec![];
    for &field in fields {
      res.push(RandomState{ field, preview: VecDeque::new(), hold: None });
    }
    for _ in 0..preview {
      let mut temp = vec![];
      for state in res {
        for piece in PIECES {
          let mut preview = state.preview.clone();
          preview.push_back(piece);
          temp.push(RandomState{ preview, ..state });
        }
      }
      res = temp;
    }
    if hold {
      let mut temp = vec![];
      for state in res {
        for piece in PIECES {
          temp.push(RandomState{ hold: Some(piece), ..state.clone() });
        }
      }
      res = temp;
    }
    res
  }

  fn next_pieces(self: &Self) -> Vec<Piece> {
    PIECES.to_vec()
  }

  fn add_piece(self: &Self, piece: Piece) -> Vec<(Self, Piece)> {
    let mut vec = vec![];
    let mut ret = self.clone();
    ret.preview.push_back(piece);
    let p = ret.preview.pop_front().unwrap();
    if let Some(hold) = self.hold {
      if hold != p {
        let mut ret = ret.clone();
        ret.hold = Some(p);
        vec.push((ret, hold));
      }
    }
    vec.push((ret, p));
    vec
  }

  fn change_field(self: &Self, field: Field) -> Self {
    RandomState{ field, ..self.clone() }
  }
  
  fn get_field(self: &Self) -> Field {
    self.field
  }
}

#[derive(Debug, Hash, Eq, PartialEq, Serialize, Deserialize, Copy, Clone, Ord, PartialOrd)]
enum Bag {
  TwoBags(usize),
  OneBag([bool; 7]),
}

// one bag will never be all true

#[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
struct BagState {
  field: Field,
  hold: Option<Piece>,
  preview: VecDeque<Piece>,
  bag: Bag,
}

impl BagState {
  fn push(self: &Self, piece: Piece) -> Self {
    let mut ret = self.clone();
    ret.preview.push_back(piece);
    match ret.bag {
      Bag::TwoBags(_) => {},
      Bag::OneBag(ref mut bag) => {
        bag[piece as usize] = true;
        if bag.iter().all(|&b| b) {
          ret.bag = Bag::TwoBags(ret.preview.len());
        }
      }
    }
    ret
  }
  fn pull(self: &Self) -> (Self, Piece) {
    let mut ret = self.clone();
    let piece = ret.preview.pop_front().unwrap();
    match ret.bag {
      Bag::TwoBags(n) => {
        if n == 1 {
          ret.bag = Bag::OneBag([false; 7]);
          match ret.bag {
            Bag::OneBag(ref mut bag) => {
              for i in ret.preview.iter() {
                bag[*i as usize] = true;
              }
            }
            _ => unreachable!()
          }
        } else {
          ret.bag = Bag::TwoBags(n - 1);
        }
      },
      Bag::OneBag(_) => {}
    }
    (ret, piece)
  }
}

impl State for BagState {
  fn new(fields: Vec<&Field>, preview: usize, hold: bool) -> Vec<Self> {
    assert!(preview <= 13);
    let mut res = vec![BagState{ field: Field([0; 4]), preview: VecDeque::new(), hold: None, bag: Bag::OneBag([false; 7]) }];
    for _ in 0..preview {
      let mut temp = vec![];
      for state in res {
        for piece in state.next_pieces() {
          temp.push(state.push(piece));
        }
      }
      res = temp;
    }
    let mut res = res.iter().map(|s| s.clone()).collect::<HashSet<_>>();
    let mut q = VecDeque::new();
    for state in res.iter() {
      q.push_back(state.clone());
    }
    while q.len() > 0 {
      let state = q.pop_front().unwrap();
      for piece in state.next_pieces() {
        let (new_state, _) = state.push(piece).pull();
        if res.contains(&new_state) {
          continue;
        }
        res.insert(new_state.clone());
        q.push_back(new_state);
      }
    }
    println!("{}", res.len());
    let mut res = res.iter().map(|s| fields.iter().map(|f| s.change_field(**f)).collect::<Vec<_>>()).flatten().collect::<Vec<_>>();
    if hold {
      let mut temp = vec![];
      for state in res {
        for piece in PIECES {
          temp.push(BagState{ hold: Some(piece), ..state.clone() });
        }
      }
      res = temp;
    }
    res
  }

  fn next_pieces(self: &Self) -> Vec<Piece> {
    match self.bag {
      Bag::TwoBags(n) => {
        let mut temp = PIECES.iter().map(|&p| p).collect::<HashSet<Piece>>();
        for p in n..self.preview.len() {
          temp.remove(&self.preview[p]);
        }
        temp.into_iter().collect::<Vec<_>>()
      }
      Bag::OneBag(ref bag) => {
        let mut res = vec![];
        for i in 0..7 {
          if !bag[i] {
            res.push(Piece::num2piece(i));
          }
        }
        res
      }
    }
  }

  fn change_field(self: &Self, field: Field) -> Self {
    BagState{ field, ..self.clone() }
  }

  fn add_piece(self: &Self, piece: Piece) -> Vec<(Self, Piece)> {
    let mut vec = vec![];
    let ret = self.push(piece);
    let (ret, p) = ret.pull();
    if let Some(hold) = self.hold {
      if hold != p {
        let mut ret = ret.clone();
        ret.hold = Some(p);
        vec.push((ret, hold));
      }
    }
    vec.push((ret, p));
    vec
  }

  fn get_field(self: &Self) -> Field {
    self.field
  }
}

struct Recorder<T> where T: Eq + std::hash::Hash + Clone {
  num2state: Vec<T>,
  state2num: HashMap<T, usize>,
  seeds: Vec<usize>,
}

impl<T> Recorder<T> where T: Eq + std::hash::Hash + Clone {
  fn new() -> Self {
    Self{ num2state: vec![], state2num: HashMap::new(), seeds: vec![] }
  }

  fn record(&mut self, state: T, position: usize) -> usize {
    if let Some(num) = self.state2num.get(&state) {
      *num
    } else {
      let num = self.num2state.len();
      self.num2state.push(state.clone());
      self.seeds.push(position);
      self.state2num.insert(state, num);
      num
    }
  }

  fn find(&self, state: &T) -> Option<usize> {
    self.state2num.get(state).map(|&num| num)
  }

  fn len(&self) -> usize {
    self.num2state.len()
  }

  fn clear(&mut self) {
    self.num2state.clear();
    self.state2num.clear();
  }
}

fn minimize_states<T>(
  states: &[T],
  continuation: &HashMap<Field, HashMap<Piece, Vec<Field>>>
) -> (Vec<usize>, Vec<Vec<Vec<usize>>>) where T: State + Eq + std::hash::Hash + Clone + Ord + Sync {
  let mut last_count = 1;
  let mut res = vec![0 as usize; states.len()];
  let mut recorder = Recorder::new();
  recorder.seeds.push(0);
  loop {
    let get_next = |state: &T| -> Vec<Vec<usize>> {
      let mut nexts = Vec::new();
      for piece in state.next_pieces() {
        let mut next = Vec::new();
        for (new_state, new_piece) in state.add_piece(piece) {
          for &field in &continuation[&new_state.get_field()][&new_piece] {
            next.push(res[states.binary_search(&new_state.change_field(field)).unwrap()]);
          }
        }
        next.sort();
        next.dedup();
        nexts.push(next);
      }
      nexts.sort();
      nexts
    };
    recorder.num2state = recorder.seeds.par_iter().map(|&seed| get_next(&states[seed])).collect();
    recorder.state2num = recorder.num2state.iter().enumerate().map(|(i, s)| (s.clone(), i)).collect();
    assert_eq!(recorder.num2state.len(), recorder.state2num.len());
    let mut new_res = vec![usize::MAX; states.len()];
    let news = new_res.iter_mut().zip(states.iter()).enumerate().par_bridge().filter_map(|(i, (num, state))| {
      let next = get_next(state);
      match recorder.find(&next) {
        Some(j) => {
          *num = j;
          None
        }
        None => Some(i)
      }
    }).collect::<Vec<_>>();
    println!("unresolved: {}", news.len());
    for i in news {
      let next = get_next(&states[i]);
      new_res[i] = recorder.record(next, i);
    }
    res = new_res;
    println!("minimized states: {}", recorder.len());
    if recorder.len() == last_count {
      return (res, recorder.num2state)
    }
    last_count = recorder.len();
    recorder.clear();
  }
}

fn main() {
  if env::args().len() < 2 {
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
        for rotated_piece in piece.get_piece() {
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
    return;
  }
  let path = env::args().nth(1).unwrap();
  let path = Path::new(&path);
  let continuation: HashMap<Field, HashMap<Piece, Vec<Field>>> = bincode::deserialize_from(
    std::io::BufReader::new(std::fs::File::open(path).unwrap()),
  ).unwrap();
  println!("{}", continuation.len());
  let mut num2state = BagState::new(continuation.keys().collect(), 6, true);
  num2state.par_sort_unstable();
  println!("{}", num2state.len());
  let (field2state, nexts) = minimize_states(&num2state, &continuation);
  let mut values = vec![0.; nexts.len()];
  let mut last_diff: f64 = 1.;
  const EPS: f64 = 1e-10;
  for i in 0.. {
    let (new_values, diffs): (Vec<_>, Vec<_>) = (0..values.len()).into_par_iter().map(|j| {
      let mut value = Mean::new();
      for next in &nexts[j] {
        let mut this_value = Max::from_value(0.);
        for &k in next {
          this_value.add(values[k] + 1.);
        }
        value.add(this_value.max());
      }
      let new_value = value.mean();
      let old_value = values[j];
      let diff = NotNan::new((new_value - old_value).abs()).unwrap();
      (new_value, diff)
    }).unzip();
    let diff = diffs.iter().max().unwrap().into_inner();
    let expected = (diff.log10() - EPS.log10()) / (last_diff.log10() - diff.log10());
    values = new_values;
    println!("{}/{:.2}: {}", i, expected + i as f64, diff);
    if diff < EPS {
      break;
    }
    last_diff = diff;
  }
  let mut states = num2state.iter().zip(field2state.iter()).map(|(state, field)| (state, values[*field])).collect::<Vec<_>>();
  states.par_sort_unstable_by_key(|&(state, _)| state.field);
  let mut values = 
    states.into_iter()
    .group_by(|(state, _)| state.field)
    .into_iter()
    .map(|(f, states)| (f, states.map(|(_, v)| v).collect::<Mean>().mean()))
    .collect::<Vec<_>>();
  values.par_sort_unstable_by_key(|&(_, value)| NotNan::new(-value).unwrap());
  for (field, value) in values.iter() {
    eprintln!("{}", value);
    print(&field, None);
  }
}
