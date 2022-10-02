use crate::states::*;

pub trait SequenceStates: States
where for<'a> <Self::State<'a> as StateProxy<'a>>::Proxy: Gen<usize>+Copy
{
  fn new(preview: usize, base_len: usize) -> Self;
}
pub struct FieldSequenceStates<S: SequenceStates>
where for<'a> <S::State<'a> as StateProxy<'a>>::Proxy: Gen<usize>+Copy
{
  fields: Vec<Field>,
  continuations: Continuation,
  base: Vec<Piece>,
  hold: bool,
  sequence: S
}
impl<S: SequenceStates> States for FieldSequenceStates<S>
where for<'a> <S::State<'a> as StateProxy<'a>>::Proxy: Gen<usize>+Copy
{
  type State<'a> = FieldSequenceState<'a, S> where Self: 'a;
  fn get_index(&self, state: &Self::State<'_>) -> Option<usize> {
    self.sequence.get_index(&state.sequence).map(|seq| self.base_len() * seq + self.fields.len() * state.hold + state.field)
  }
  fn get_state(&self, index: usize) -> Option<Self::State<'_>> {
    let (sequence, field_hold) = index.div_rem(&self.base_len());
    let (hold, field) = field_hold.div_rem(&self.fields.len());
    self.sequence.get_state(sequence).map(|sequence| FieldSequenceState { field, hold, sequence })
  }
}
impl<S: SequenceStates> HasLength for FieldSequenceStates<S>
where for<'a> <S::State<'a> as StateProxy<'a>>::Proxy: Gen<usize>+Copy
{
  fn len(&self) -> usize {
    self.base_len() * self.sequence.len()
  }
}
impl<'b, S: SequenceStates> Creatable<'b> for FieldSequenceStates<S>
where for<'a> <S::State<'a> as StateProxy<'a>>::Proxy: Gen<usize>+Copy
{
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
impl<S: SequenceStates> FieldSequenceStates<S>
where for<'a> <S::State<'a> as StateProxy<'a>>::Proxy: Gen<usize>+Copy
{
  fn base_len(&self) -> usize {
    if self.hold {
      self.fields.len() * self.base.len()
    } else {
      self.fields.len()
    }
  }
}

pub struct FieldSequenceState<'s, S: SequenceStates+'s>
where for<'a> <S::State<'a> as StateProxy<'a>>::Proxy: Gen<usize>+Copy
{
  field: usize,
  hold: usize,
  sequence: S::State<'s>
}
impl<'s, S: SequenceStates> StateProxy<'s> for FieldSequenceState<'s, S>
where for<'a> <S::State<'a> as StateProxy<'a>>::Proxy: Gen<usize>+Copy
{
  type RealStates = FieldSequenceStates<S>;
  type Branch = <S::State<'s> as StateProxy<'s>>::Branch;
  type BranchIter = <S::State<'s> as StateProxy<'s>>::BranchIter;
  type Proxy = Self;
  type SelfIter = std::vec::IntoIter<Self::Proxy>;
  fn next_pieces(self, states: &'s Self::RealStates) -> Self::BranchIter {
    self.sequence.next_pieces(&states.sequence)
  }
  fn next_states(self, states: &'s Self::RealStates, piece: Self::Branch) -> Self::SelfIter {
    let indices = &states.continuations.cont_index[self.field];
    self
      .sequence
      .next_states(&states.sequence, piece)
      .flat_map(move |p| {
        let current = Gen::<usize>::gen(p);
        let (left, right) = indices[states.base[current] as usize];
        let sequence = Gen::<S::State<'s>>::gen(p);
        states.continuations.continuations[left..right]
          .iter()
          .map(move |&field| FieldSequenceState {
            field,
            hold: self.hold,
            sequence
          })
          .chain({
            let (left, right) = indices[states.base[self.hold] as usize];
            states.continuations.continuations[left..right].iter().filter(|_| states.hold).map(
              move |&field| FieldSequenceState {
                field,
                hold: current,
                sequence
              }
            )
          })
      })
      .collect_vec()
      .into_iter()
  }
}
impl<'s, S: SequenceStates> Clone for FieldSequenceState<'s, S>
where for<'a> <S::State<'a> as StateProxy<'a>>::Proxy: Gen<usize>+Copy
{
  fn clone(&self) -> Self {
    Self { ..*self }
  }
}
impl<'s, S: SequenceStates> Copy for FieldSequenceState<'s, S> where for<'a> <S::State<'a> as StateProxy<'a>>::Proxy: Gen<usize>+Copy
{}

pub struct RandomSequenceStates {
  preview: usize,
  base_len: usize
}
impl SequenceStates for RandomSequenceStates {
  fn new(preview: usize, base_len: usize) -> Self {
    Self { preview, base_len }
  }
}
impl States for RandomSequenceStates {
  type State<'a> = RandomSequenceState;
  fn get_index(&self, state: &Self::State<'_>) -> Option<usize> {
    Some(state.index)
  }
  fn get_state(&self, index: usize) -> Option<Self::State<'_>> {
    (index < self.len()).then_some(RandomSequenceState { index })
  }
}
impl HasLength for RandomSequenceStates {
  fn len(&self) -> usize {
    self.base_len.pow(self.preview as u32)
  }
}

#[derive(Clone, Copy)]
pub struct RandomSequenceState {
  index: usize
}
impl<'a> StateProxy<'a> for RandomSequenceState {
  type RealStates = RandomSequenceStates;
  type Branch = usize;
  type BranchIter = std::ops::Range<usize>;
  type Proxy = (Self, usize);
  type SelfIter = std::iter::Once<Self::Proxy>;
  fn next_pieces(self, states: &'a Self::RealStates) -> Self::BranchIter {
    0..states.base_len
  }
  fn next_states(self, states: &'a Self::RealStates, piece: Self::Branch) -> Self::SelfIter {
    std::iter::once(if states.preview < 1 {
      (self, piece)
    } else {
      let (mut index, current) = self.index.div_rem(&states.base_len);
      index += piece * states.base_len.pow((states.preview - 1) as u32);
      (Self { index }, current)
    })
  }
}

impl<T> Gen<usize> for (T, usize) {
  fn gen(self) -> usize {
    self.1
  }
}
impl<'a, T: StateProxy<'a>> Gen<T> for (T, usize) {
  fn gen(self) -> T {
    self.0
  }
}

pub struct BagSequenceStates {
  continuation: Continuation<(usize, usize)>
}
impl SequenceStates for BagSequenceStates {
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
      type Item = std::vec::IntoIter<std::iter::Once<(usize, usize)>>;
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
              std::iter::once((self.find_or_insert(top), piece))
            })
            .collect_vec()
            .into_iter()
        })
      }
    }
    let mut bfs = BFS { mapping: Default::default(), inverse: Default::default(), base_len };
    bfs.find_or_insert({
      let seq: VecDeque<usize> = (0..preview).map(|i| i % base_len).collect();
      let available = (0..base_len).map(|i| i > seq.back().copied().unwrap_or(base_len)).collect();
      (seq, available)
    });
    Self { continuation: bfs.collect() }
  }
}
impl States for BagSequenceStates {
  type State<'a> = BagSequenceState;
  fn get_index(&self, state: &Self::State<'_>) -> Option<usize> {
    (state.index < self.len()).then(|| state.index)
  }
  fn get_state(&self, index: usize) -> Option<Self::State<'_>> {
    (index < self.len()).then(|| Self::State { index })
  }
}
impl HasLength for BagSequenceStates {
  fn len(&self) -> usize {
    self.continuation.len()
  }
}

#[derive(Clone, Copy)]
pub struct BagSequenceState {
  index: usize
}
impl<'a> StateProxy<'a> for BagSequenceState {
  type RealStates = BagSequenceStates;
  type Branch = usize;
  type BranchIter = std::ops::Range<usize>;
  type Proxy = (Self, usize);
  type SelfIter = std::iter::Once<Self::Proxy>;
  fn next_pieces(self, states: &'a Self::RealStates) -> Self::BranchIter {
    0..states.continuation.cont_index[self.index].len()
  }
  fn next_states(self, states: &'a Self::RealStates, piece: Self::Branch) -> Self::SelfIter {
    let (target, current) =
      states.continuation.continuations[states.continuation.cont_index[self.index][piece].0];
    std::iter::once((Self { index: target }, current))
  }
}
