use crate::states::*;

pub trait SequenceStates: States
where for<'a> <Self::State<'a> as StateProxy>::Proxy: Into<Piece>+Copy
{
  fn new(preview: usize, hold: bool) -> Self;
}
pub struct FieldSequenceStates<S: SequenceStates>
where for<'a> <S::State<'a> as StateProxy>::Proxy: Into<Piece>+Copy
{
  fields: Vec<Field>,
  continuations: Continuation,
  sequence: S
}
impl<S: SequenceStates> States for FieldSequenceStates<S>
where for<'a> <S::State<'a> as StateProxy>::Proxy: Into<Piece>+Copy
{
  type State<'a> = FieldSequenceState<'a, S> where Self: 'a;
  fn get_index(&self, state: &Self::State<'_>) -> Option<usize> {
    self.sequence.get_index(&state.sequence).map(|seq| self.fields.len() * seq + state.field)
  }
  fn get_state(&self, index: usize) -> Option<Self::State<'_>> {
    let (sequence, field) = index.div_rem(&self.fields.len());
    self.sequence.get_state(sequence).map(|sequence| FieldSequenceState {
      states: self,
      field,
      sequence
    })
  }
}
impl<S: SequenceStates> HasLength for FieldSequenceStates<S>
where for<'a> <S::State<'a> as StateProxy>::Proxy: Into<Piece>+Copy
{
  fn len(&self) -> usize {
    self.fields.len() * self.sequence.len()
  }
}
impl<'b, S: SequenceStates> Creatable<'b> for FieldSequenceStates<S>
where for<'a> <S::State<'a> as StateProxy>::Proxy: Into<Piece>+Copy
{
  fn new(
    continuations: &'b HashMap<Field, HashMap<Piece, Vec<Field>>>,
    preview: usize,
    hold: bool
  ) -> Self {
    let (fields, continuations) = Continuation::new(continuations);
    Self { fields, continuations, sequence: S::new(preview, hold) }
  }
}

pub struct FieldSequenceState<'s, S: SequenceStates>
where for<'a> <S::State<'a> as StateProxy>::Proxy: Into<Piece>+Copy
{
  states: &'s FieldSequenceStates<S>,
  field: usize,
  sequence: S::State<'s>
}
impl<'s, S: SequenceStates> StateProxy for FieldSequenceState<'s, S>
where for<'a> <S::State<'a> as StateProxy>::Proxy: Into<Piece>+Copy
{
  type Branch = <S::State<'s> as StateProxy>::Branch;
  type BranchIter = <S::State<'s> as StateProxy>::BranchIter;
  type Proxy = Self;
  type SelfIter = std::vec::IntoIter<Self::Proxy>;
  fn next_pieces(&self) -> Self::BranchIter {
    self.sequence.next_pieces()
  }
  fn next_states(&self, piece: Self::Branch) -> Self::SelfIter {
    let indices = &self.states.continuations.cont_index[self.field];
    self.sequence.next_states(piece).flat_map(|p| {
      let (left, right) = indices[Into::<Piece>::into(p) as usize];
      self.states.continuations.continuations[left..right].iter().map(move |&field| {
        FieldSequenceState { field, sequence: Into::<S::State<'s>>::into(p), ..*self }
      })
    })
    .collect_vec()
    .into_iter()
  }
}

pub struct RandomSequenceStates {
  preview: usize,
  hold: bool,
  base: Vec<Piece>
}
impl SequenceStates for RandomSequenceStates {
  fn new(preview: usize, hold: bool) -> Self {
    Self { preview, hold, base: PIECES.iter().cloned().collect() }
  }
}
impl States for RandomSequenceStates {
  type State<'a> = RandomSequenceState<'a>;
  fn get_index(&self, state: &Self::State<'_>) -> Option<usize> {
    Some(state.index)
  }
  fn get_state(&self, index: usize) -> Option<Self::State<'_>> {
    (index < self.len()).then_some(RandomSequenceState { states: self, index })
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
pub struct RandomSequenceState<'a> {
  states: &'a RandomSequenceStates,
  index: usize
}
impl<'a> StateProxy for RandomSequenceState<'a> {
  type Branch = usize;
  type BranchIter = std::ops::Range<usize>;
  type Proxy = RandomSequenceStateWithCurrent<'a>;
  type SelfIter = arrayvec::IntoIter<Self::Proxy, 2>;
  fn next_pieces(&self) -> Self::BranchIter {
    0..self.states.base.len()
  }
  fn next_states(&self, piece: Self::Branch) -> Self::SelfIter {
    let mut possibilities = ArrayVec::new();
    if self.states.seq_len() < 1 {
      possibilities.push(Self::Proxy { state: *self, current: piece })
    } else {
      let (mut index, current) = self.index.div_rem(&self.states.base.len());
      index += piece * self.states.base.len().pow((self.states.seq_len() - 1) as u32);
      possibilities.push(Self::Proxy { state: Self { states: self.states, index }, current });
      if self.states.hold {
        let swap = index % self.states.base.len();
        index -= swap;
        index += current;
        possibilities
          .push(Self::Proxy { state: Self { states: self.states, index }, current: swap })
      }
    }
    possibilities.into_iter()
  }
}

#[derive(Clone, Copy)]
pub struct RandomSequenceStateWithCurrent<'a> {
  state: RandomSequenceState<'a>,
  current: usize
}
impl<'a> Into<Piece> for RandomSequenceStateWithCurrent<'a> {
  fn into(self) -> Piece {
    self.state.states.base[self.current]
  }
}
impl<'a> Into<RandomSequenceState<'a>> for RandomSequenceStateWithCurrent<'a> {
  fn into(self) -> RandomSequenceState<'a> {
    self.state
  }
}
