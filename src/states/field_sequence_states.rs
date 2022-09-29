use crate::states::*;

pub trait SequenceStates: States
where for<'a> <Self::State<'a> as StateProxy<'a>>::Proxy: Into<Piece>+Copy
{
  fn new(preview: usize, hold: bool) -> Self;
}
pub struct FieldSequenceStates<S: SequenceStates>
where for<'a> <S::State<'a> as StateProxy<'a>>::Proxy: Into<Piece>+Copy
{
  fields: Vec<Field>,
  continuations: Continuation,
  sequence: S
}
impl<S: SequenceStates> States for FieldSequenceStates<S>
where for<'a> <S::State<'a> as StateProxy<'a>>::Proxy: Into<Piece>+Copy
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
where for<'a> <S::State<'a> as StateProxy<'a>>::Proxy: Into<Piece>+Copy
{
  fn len(&self) -> usize {
    self.fields.len() * self.sequence.len()
  }
}
impl<'b, S: SequenceStates> Creatable<'b> for FieldSequenceStates<S>
where for<'a> <S::State<'a> as StateProxy<'a>>::Proxy: Into<Piece>+Copy
{
  fn new(
    continuations: &'b HashMap<Field, HashMap<Piece, Vec<Field>>>,
    preview: usize,
    hold: bool
  ) -> Self {
    let (fields, continuations) = Continuation::new(continuations);
    let sequence = S::new(preview, hold);
    assert!(
      (sequence.len() as f64).log2() + (fields.len() as f64).log2() <= (usize::MAX as f64).log2()
    );
    Self { fields, continuations, sequence }
  }
}

pub struct FieldSequenceState<'s, S: SequenceStates+'s>
where for<'a> <S::State<'a> as StateProxy<'a>>::Proxy: Into<Piece>+Copy
{
  field: usize,
  sequence: S::State<'s>
}
impl<'s, S: SequenceStates> StateProxy<'s> for FieldSequenceState<'s, S>
where for<'a> <S::State<'a> as StateProxy<'a>>::Proxy: Into<Piece>+Copy
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
        let (left, right) = indices[Into::<Piece>::into(p) as usize];
        let sequence = Into::<S::State<'s>>::into(p);
        states.continuations.continuations[left..right]
          .iter()
          .map(move |&field| FieldSequenceState { field, sequence })
      })
      .collect_vec()
      .into_iter()
  }
}
impl<'s, S: SequenceStates> Clone for FieldSequenceState<'s, S>
where for<'a> <S::State<'a> as StateProxy<'a>>::Proxy: Into<Piece>+Copy
{
  fn clone(&self) -> Self {
    Self { ..*self }
  }
}
impl<'s, S: SequenceStates> Copy for FieldSequenceState<'s, S> where for<'a> <S::State<'a> as StateProxy<'a>>::Proxy: Into<Piece>+Copy
{}

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
  type Proxy = RandomSequenceStateWithCurrent;
  type SelfIter = arrayvec::IntoIter<Self::Proxy, 2>;
  fn next_pieces(self, states: &'a Self::RealStates) -> Self::BranchIter {
    0..states.base.len()
  }
  fn next_states(self, states: &'a Self::RealStates, piece: Self::Branch) -> Self::SelfIter {
    let mut possibilities = ArrayVec::new();
    if states.seq_len() < 1 {
      possibilities.push(Self::Proxy { state: self, current: states.base[piece] })
    } else {
      let (mut index, current) = self.index.div_rem(&states.base.len());
      index += piece * states.base.len().pow((states.seq_len() - 1) as u32);
      possibilities.push(Self::Proxy { state: Self { index }, current: states.base[current] });
      if states.hold {
        let swap = index % states.base.len();
        index -= swap;
        index += current;
        possibilities.push(Self::Proxy { state: Self { index }, current: states.base[swap] })
      }
    }
    possibilities.into_iter()
  }
}

#[derive(Clone, Copy)]
pub struct RandomSequenceStateWithCurrent {
  state: RandomSequenceState,
  current: Piece
}
impl Into<Piece> for RandomSequenceStateWithCurrent {
  fn into(self) -> Piece {
    self.current
  }
}
impl Into<RandomSequenceState> for RandomSequenceStateWithCurrent {
  fn into(self) -> RandomSequenceState {
    self.state
  }
}
