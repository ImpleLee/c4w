use crate::states::*;

pub trait SequenceStates: HasLength+std::marker::Sync {
  type State: Copy;
  type Proxy: StateWithPiece<Self::State>;
  type ProxyIter<'a>: Iterator<Item=Self::Proxy> where Self: 'a;
  fn new(preview: usize, base_len: usize) -> Self;
  fn get_state(&self, index: usize) -> Option<Self::State>;
  fn get_index(&self, state: &Self::State) -> Option<usize>;
  fn next_pieces(&self, state: Self::State) -> Self::ProxyIter<'_>;
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

pub struct FieldSequenceStates<S: SequenceStates> {
  fields: Vec<Field>,
  continuations: Continuation,
  base: Vec<Piece>,
  hold: bool,
  sequence: S
}
impl<S: SequenceStates> States for FieldSequenceStates<S> {
  type State = FieldSequenceState<S>;
  type StateIter<'a> = std::vec::IntoIter<Self::State> where S: 'a;
  type Branch = (usize, usize, S::Proxy);
  type BranchIter<'a> = Box<dyn Iterator<Item=Self::Branch>+'a> where S: 'a;
  fn get_index(&self, state: &Self::State) -> Option<usize> {
    self
      .sequence
      .get_index(&state.sequence)
      .map(|seq| self.base_len() * seq + self.fields.len() * state.hold + state.field)
  }
  fn get_state(&self, index: usize) -> Option<Self::State> {
    let (sequence, field_hold) = index.div_rem(&self.base_len());
    let (hold, field) = field_hold.div_rem(&self.fields.len());
    self.sequence.get_state(sequence).map(|sequence| FieldSequenceState { field, hold, sequence })
  }
  fn next_pieces(&self, state: Self::State) -> Self::BranchIter<'_> {
    Box::new(self.sequence.next_pieces(state.sequence).map(move |v| (state.field, state.hold, v)))
    //.collect_vec()
    //.into_iter()
  }
  fn next_states(&self, (field, hold, piece): Self::Branch) -> Self::StateIter<'_> {
    let indices = &self.continuations.cont_index[field];
    let sequence = piece.gen_state();
    let current = piece.gen_piece();
    let (left, right) = indices[self.base[current] as usize];
    self.continuations.continuations[left..right]
      .iter()
      .map(move |&field| FieldSequenceState { field, hold, sequence })
      .chain({
        let (left, right) = indices[self.base[hold] as usize];
        self.continuations.continuations[left..right]
          .iter()
          .filter(|_| self.hold)
          .map(move |&field| FieldSequenceState { field, hold: current, sequence })
      })
      .collect_vec()
      .into_iter()
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
    let base: Vec<_> = PIECES.iter().cloned().collect();
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

pub struct FieldSequenceState<S: SequenceStates> {
  field: usize,
  hold: usize,
  sequence: S::State
}

pub struct RandomSequenceStates {
  preview: usize,
  base_len: usize
}
impl SequenceStates for RandomSequenceStates {
  type State = usize;
  type Proxy = (Self::State, usize);
  type ProxyIter<'a> = std::vec::IntoIter<Self::Proxy>;
  fn new(preview: usize, base_len: usize) -> Self {
    Self { preview, base_len }
  }
  fn get_index(&self, state: &Self::State) -> Option<usize> {
    Some(*state)
  }
  fn get_state(&self, index: usize) -> Option<Self::State> {
    Some(index)
  }
  fn next_pieces(&self, mut state: Self::State) -> Self::ProxyIter<'_> {
    (0..self.base_len)
      .map(|piece| {
        state += piece * self.base_len.pow(self.preview as u32);
        state.div_rem(&self.base_len)
      })
      .collect_vec()
      .into_iter()
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
  type ProxyIter<'a> = std::iter::Cloned<std::slice::Iter<'a, Self::Proxy>>;
  fn new(preview: usize, base_len: usize) -> Self {
    type State = (VecDeque<usize>, Vec<bool>);
    struct BFS {
      mapping: HashMap<State, usize>,
      inverse: VecDeque<State>,
      base_len: usize
    }
    impl BFS {
      fn find_or_insert(&mut self, state: State) -> usize {
        let len = self.mapping.len();
        *self.mapping.entry(state.clone()).or_insert_with(|| {
          let v = len;
          self.inverse.push_back(state);
          v
        })
      }
    }
    impl Iterator for BFS {
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
    let mut bfs = BFS { mapping: Default::default(), inverse: Default::default(), base_len };
    bfs.find_or_insert({
      let seq: VecDeque<usize> = (0..preview).map(|i| i % base_len).collect();
      let available = (0..base_len).map(|i| i > seq.back().copied().unwrap_or(base_len)).collect();
      (seq, available)
    });
    bfs.collect()
  }
  fn get_index(&self, state: &Self::State) -> Option<usize> {
    Some(*state)
  }
  fn get_state(&self, index: usize) -> Option<Self::State> {
    Some(index)
  }
  fn next_pieces(&self, state: Self::State) -> Self::ProxyIter<'_> {
    self[state].iter().cloned()
  }
}
impl HasLength for BagSequenceStates {
  fn len(&self) -> usize {
    self.len()
  }
}
