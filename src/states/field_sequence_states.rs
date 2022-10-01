use crate::states::*;

pub trait SequenceStates: States
where for<'a> <Self::State<'a> as StateProxy<'a>>::Proxy: Gen<Piece>+Copy
{
  fn new(preview: usize, hold: bool, base: &[Piece]) -> Self;
}
pub struct FieldSequenceStates<S: SequenceStates>
where for<'a> <S::State<'a> as StateProxy<'a>>::Proxy: Gen<Piece>+Copy
{
  fields: Vec<Field>,
  continuations: Continuation,
  sequence: S
}
impl<S: SequenceStates> States for FieldSequenceStates<S>
where for<'a> <S::State<'a> as StateProxy<'a>>::Proxy: Gen<Piece>+Copy
{
  type State<'a> = FieldSequenceState<'a, S> where Self: 'a;
  fn get_index(&self, state: &Self::State<'_>) -> Option<usize> {
    self.sequence.get_index(&state.sequence).map(|seq| self.fields.len() * seq + state.field)
  }
  fn get_state(&self, index: usize) -> Option<Self::State<'_>> {
    let (sequence, field) = index.div_rem(&self.fields.len());
    self.sequence.get_state(sequence).map(|sequence| FieldSequenceState { field, sequence })
  }
}
impl<S: SequenceStates> HasLength for FieldSequenceStates<S>
where for<'a> <S::State<'a> as StateProxy<'a>>::Proxy: Gen<Piece>+Copy
{
  fn len(&self) -> usize {
    self.fields.len() * self.sequence.len()
  }
}
impl<'b, S: SequenceStates> Creatable<'b> for FieldSequenceStates<S>
where for<'a> <S::State<'a> as StateProxy<'a>>::Proxy: Gen<Piece>+Copy
{
  fn new(
    continuations: &'b HashMap<Field, HashMap<Piece, Vec<Field>>>,
    preview: usize,
    hold: bool
  ) -> Self {
    let (fields, continuations) = Continuation::new(continuations);
    let sequence = S::new(preview, hold, &PIECES);
    assert!(
      (sequence.len() as f64).log2() + (fields.len() as f64).log2() <= (usize::MAX as f64).log2()
    );
    Self { fields, continuations, sequence }
  }
}

pub struct FieldSequenceState<'s, S: SequenceStates+'s>
where for<'a> <S::State<'a> as StateProxy<'a>>::Proxy: Gen<Piece>+Copy
{
  field: usize,
  sequence: S::State<'s>
}
impl<'s, S: SequenceStates> StateProxy<'s> for FieldSequenceState<'s, S>
where for<'a> <S::State<'a> as StateProxy<'a>>::Proxy: Gen<Piece>+Copy
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
        let (left, right) = indices[Gen::<Piece>::gen(p) as usize];
        let sequence = Gen::<S::State<'s>>::gen(p);
        states.continuations.continuations[left..right]
          .iter()
          .map(move |&field| FieldSequenceState { field, sequence })
      })
      .collect_vec()
      .into_iter()
  }
}
impl<'s, S: SequenceStates> Clone for FieldSequenceState<'s, S>
where for<'a> <S::State<'a> as StateProxy<'a>>::Proxy: Gen<Piece>+Copy
{
  fn clone(&self) -> Self {
    Self { ..*self }
  }
}
impl<'s, S: SequenceStates> Copy for FieldSequenceState<'s, S> where for<'a> <S::State<'a> as StateProxy<'a>>::Proxy: Gen<Piece>+Copy
{}

pub struct RandomSequenceStates {
  preview: usize,
  hold: bool,
  base: Vec<Piece>
}
impl SequenceStates for RandomSequenceStates {
  fn new(preview: usize, hold: bool, base: &[Piece]) -> Self {
    Self { preview, hold, base: base.iter().cloned().collect() }
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
    self.base.len().pow(self.seq_len() as u32)
  }
}
impl RandomSequenceStates {
  fn seq_len(&self) -> usize {
    self.preview + self.hold as usize
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
  type Proxy = (Self, Piece);
  type SelfIter = arrayvec::IntoIter<Self::Proxy, 2>;
  fn next_pieces(self, states: &'a Self::RealStates) -> Self::BranchIter {
    0..states.base.len()
  }
  fn next_states(self, states: &'a Self::RealStates, piece: Self::Branch) -> Self::SelfIter {
    let mut possibilities = ArrayVec::new();
    if states.seq_len() < 1 {
      possibilities.push((self, states.base[piece]))
    } else {
      let (mut index, current) = self.index.div_rem(&states.base.len());
      index += piece * states.base.len().pow((states.seq_len() - 1) as u32);
      possibilities.push((Self { index }, states.base[current]));
      if states.hold {
        let swap = index % states.base.len();
        index -= swap;
        index += current;
        possibilities.push((Self { index }, states.base[swap]))
      }
    }
    possibilities.into_iter()
  }
}

impl<T> Gen<Piece> for (T, Piece) {
  fn gen(self) -> Piece {
    self.1
  }
}
impl<'a, T: StateProxy<'a>> Gen<T> for (T, Piece) {
  fn gen(self) -> T {
    self.0
  }
}

pub struct BagSequenceStates {
  base: Vec<Piece>,
  hold: bool,
  continuation: Continuation<(usize, usize)>
}
impl SequenceStates for BagSequenceStates {
  fn new(preview: usize, hold: bool, base: &[Piece]) -> Self {
    type State = (VecDeque<usize>, Vec<bool>);
    let next = |mut state: State| -> Box<dyn Iterator<Item=(usize, State)>> {
      fn push(state: &mut State, i: usize) {
        assert!(state.1[i]);
        state.0.push_back(i);
        state.1[i] = false;
      }
      fn pop(state: &mut State) -> usize {
        state.0.pop_front().unwrap()
      }
      if state.1.iter().all(|available| !available) {
        state.1 = (0..base.len()).map(|_| true).collect();
      }
      Box::new(state.1.clone().into_iter().enumerate().filter(|(_, b)| *b).map(move |(i, _)| {
        let mut state = state.clone();
        push(&mut state, i);
        (pop(&mut state), state)
      }))
    };
    #[derive(Default)]
    struct Vocab<T: std::hash::Hash+Clone+Eq> {
      mapping: HashMap<T, usize>,
      inverse: Vec<T>
    }
    impl<T: std::hash::Hash+Clone+Eq> Vocab<T> {
      fn find_or_insert(&mut self, state: T) -> usize {
        *self.mapping.entry(state.clone()).or_insert_with(|| {
          let v = self.inverse.len();
          self.inverse.push(state);
          v
        })
      }
      fn find(&self, state: &T) -> Option<usize> {
        self.mapping.get(state).copied()
      }
      fn len(&self) -> usize {
        self.inverse.len()
      }
    }
    struct BFS<F: Fn(State) -> Box<dyn Iterator<Item=(usize, State)>>> {
      vocab: Vocab<State>,
      processed: usize,
      my_next: F
    }
    impl<F: Fn(State) -> Box<dyn Iterator<Item=(usize, State)>>> Iterator for BFS<F> {
      type Item = std::vec::IntoIter<std::iter::Once<(usize, usize)>>;
      fn next(&mut self) -> Option<Self::Item> {
        (self.processed < self.vocab.len()).then(move || {
          let top = self.vocab.inverse[self.processed].clone();
          self.processed += 1;
          (self.my_next)(top)
            .map(move |(piece, state)| std::iter::once((self.vocab.find_or_insert(state), piece)))
            .collect_vec()
            .into_iter()
        })
      }
    }
    let mut bfs = BFS { vocab: Vocab::default(), processed: 0, my_next: next };
    bfs.vocab.find_or_insert({
      let seq: VecDeque<usize> = (0..preview).map(|i| i % base.len()).collect();
      let available =
        (0..base.len()).map(|i| i > seq.back().copied().unwrap_or(base.len())).collect();
      (seq, available)
    });
    Self { base: base.iter().cloned().collect(), hold, continuation: bfs.collect() }
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
    if self.hold { self.continuation.len() * self.base.len() } else { self.continuation.len() }
  }
}

#[derive(Clone, Copy)]
pub struct BagSequenceState {
  index: usize,
}
impl<'a> StateProxy<'a> for BagSequenceState {
  type RealStates = BagSequenceStates;
  type Branch = usize;
  type BranchIter = std::ops::Range<usize>;
  type Proxy = (Self, Piece);
  type SelfIter = arrayvec::IntoIter<Self::Proxy, 2>;
  fn next_pieces(self, states: &'a Self::RealStates) -> Self::BranchIter {
    0..states.continuation.cont_index[(if states.hold { self.index / states.base.len() } else { self.index })].len()
  }
  fn next_states(self, states: &'a Self::RealStates, piece: Self::Branch) -> Self::SelfIter {
    let mut av = ArrayVec::new();
    if states.hold {
      let (index, hold) = self.index.div_rem(&states.base.len());
      let (target, current) = states.continuation.continuations[states.continuation.cont_index[index][piece].0];
      av.push((Self { index: target*states.base.len() + current }, states.base[hold]));
      av.push((Self { index: target*states.base.len() + hold }, states.base[current]));
    } else {
      let (target, current) = states.continuation.continuations[states.continuation.cont_index[self.index][piece].0];
      av.push((Self { index: target }, states.base[current]));
    }
    av.into_iter()
  }
}
