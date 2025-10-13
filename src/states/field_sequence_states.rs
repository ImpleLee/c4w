use crate::states::*;

pub trait SequenceStates: HasLength+std::marker::Sync+serde::Serialize {
  type State: Copy;
  type Proxy: StateWithPiece<Self::State>;
  fn new(preview: usize, base_len: usize) -> Self;
  fn decode(&self, index: usize) -> Option<Self::State>;
  fn encode(&self, state: &Self::State) -> Option<usize>;
  fn next_pieces(&self, state: Self::State) -> impl Iterator<Item=Self::Proxy>;
}
pub trait StateWithPiece<T> {
  fn gen_state(&self) -> T;
  fn gen_piece(&self) -> usize;
}
impl<T: Clone> StateWithPiece<T> for (T, usize) {
  fn gen_state(&self) -> T {
    self.0.clone()
  }
  fn gen_piece(&self) -> usize {
    self.1
  }
}

#[derive(serde::Serialize)]
pub struct FieldSequenceStates<S: SequenceStates> {
  fields: Vec<Field>,
  continuations: Continuation,
  base: Vec<Piece>,
  hold: bool,
  sequence: S
}
impl<S: SequenceStates> States for FieldSequenceStates<S> {
  type State = (usize, usize, S::State);
  type Branch = (usize, usize, S::Proxy);
  fn encode(&self, &(field, hold, sequence): &Self::State) -> Option<usize> {
    self
      .sequence
      .encode(&sequence)
      .map(|seq| self.base_len() * seq + self.fields.len() * hold + field)
  }
  fn decode(&self, index: usize) -> Option<Self::State> {
    let (sequence, field_hold) = index.div_rem(&self.base_len());
    let (hold, field) = field_hold.div_rem(&self.fields.len());
    self.sequence.decode(sequence).map(|sequence| (field, hold, sequence))
  }
  fn next_pieces(&self, (field, hold, sequence): Self::State) -> impl Iterator<Item=Self::Branch> {
    self.sequence.next_pieces(sequence).map(move |p| (field, hold, p))
  }
  fn next_states(&self, (field, hold, piece): Self::Branch) -> impl Iterator<Item=Self::State> {
    let indices = &self.continuations.cont_index[field];
    let sequence = piece.gen_state();
    let current = piece.gen_piece();
    let (left, right) = indices[self.base[current] as usize];
    self.continuations.continuations[left..right]
      .iter()
      .map(move |&field| (field, hold, sequence))
      .chain({
        let (left, right) = if self.hold { indices[self.base[hold] as usize] } else { (0, 0) };
        self.continuations.continuations[left..right]
          .iter()
          .map(move |&field| (field, current, sequence))
      })
  }
}
impl<S: SequenceStates> HasLength for FieldSequenceStates<S> {
  fn len(&self) -> usize {
    self.base_len() * self.sequence.len()
  }
}
impl<'b, S: SequenceStates> Creatable<'b> for FieldSequenceStates<S> {
  fn new(
    continuations: &'b HashMap<Field, HashMap<Piece, Vec<Field>>>,
    preview: usize,
    hold: bool
  ) -> Self {
    let base: Vec<_> = PIECES.to_vec();
    let (fields, continuations) = Continuation::new(continuations);
    let sequence = S::new(preview, base.len());
    assert!(
      (sequence.len() as f64).log2()
        + (fields.len() as f64).log2()
        + (if hold { base.len() } else { 1 } as f64).log2()
        <= (usize::MAX as f64).log2()
    );
    Self { fields, continuations, sequence, hold, base }
  }
}
impl<S: SequenceStates> FieldSequenceStates<S> {
  fn base_len(&self) -> usize {
    if self.hold {
      self.fields.len() * self.base.len()
    } else {
      self.fields.len()
    }
  }
}

#[derive(serde::Serialize)]
pub struct RandomSequenceStates {
  preview: usize,
  base_len: usize
}
impl SequenceStates for RandomSequenceStates {
  type State = usize;
  type Proxy = (Self::State, usize);
  fn new(preview: usize, base_len: usize) -> Self {
    Self { preview, base_len }
  }
  fn encode(&self, state: &Self::State) -> Option<usize> {
    Some(*state)
  }
  fn decode(&self, index: usize) -> Option<Self::State> {
    Some(index)
  }
  fn next_pieces(&self, state: Self::State) -> impl Iterator<Item=Self::Proxy> {
    (0..self.base_len)
      .map(move |piece| {
        let state = state + piece * self.base_len.pow(self.preview as u32);
        state.div_rem(&self.base_len)
      })
  }
}
impl HasLength for RandomSequenceStates {
  fn len(&self) -> usize {
    self.base_len.pow(self.preview as u32)
  }
}

pub type BagSequenceStates = Vec<ArrayVec<(usize, usize), 7>>;
impl SequenceStates for BagSequenceStates {
  type State = usize;
  type Proxy = (Self::State, usize);
  fn new(preview: usize, base_len: usize) -> Self {
    type State = (VecDeque<usize>, Vec<bool>);
    struct Bfs {
      mapping: HashMap<State, usize>,
      inverse: VecDeque<State>,
      base_len: usize
    }
    impl Bfs {
      fn find_or_insert(&mut self, state: State) -> usize {
        let len = self.mapping.len();
        *self.mapping.entry(state.clone()).or_insert_with(|| {
          let v = len;
          self.inverse.push_back(state);
          v
        })
      }
    }
    impl Iterator for Bfs {
      type Item = ArrayVec<(usize, usize), 7>;
      fn next(&mut self) -> Option<Self::Item> {
        (!self.inverse.is_empty()).then(move || {
          let mut top = self.inverse.pop_front().unwrap();
          if top.1.iter().all(|available| !available) {
            top.1 = (0..self.base_len).map(|_| true).collect();
          }
          top
            .1
            .clone()
            .into_iter()
            .enumerate()
            .filter(|(_, b)| *b)
            .map(move |(i, _)| {
              let mut top = top.clone();
              top.1[i] = false;
              top.0.push_back(i);
              let piece = top.0.pop_front().unwrap();
              (self.find_or_insert(top), piece)
            })
            .collect()
        })
      }
    }
    let mut bfs = Bfs { mapping: Default::default(), inverse: Default::default(), base_len };
    bfs.find_or_insert({
      let seq: VecDeque<usize> = (0..preview).map(|i| i % base_len).collect();
      let available = (0..base_len).map(|i| i > seq.back().copied().unwrap_or(base_len)).collect();
      (seq, available)
    });
    bfs.collect()
  }
  fn encode(&self, state: &Self::State) -> Option<usize> {
    Some(*state)
  }
  fn decode(&self, index: usize) -> Option<Self::State> {
    Some(index)
  }
  fn next_pieces(&self, state: Self::State) -> impl Iterator<Item=Self::Proxy> {
    self[state].iter().cloned()
  }
}
impl HasLength for BagSequenceStates {
  fn len(&self) -> usize {
    self.len()
  }
}
