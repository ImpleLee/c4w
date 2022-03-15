mod random_states;
pub use random_states::*;

use std::collections::HashMap;
use crate::basics::{Field, Piece};

pub trait StateProxy {
	type Branch;
	type MarkovState: std::fmt::Display + Ord + PartialEq + Clone + Send;
	type BranchIter: Iterator<Item=Self::Branch>;
	type SelfIter: Iterator<Item=Self>;
	fn next_pieces(self: &Self) -> Self::BranchIter;
	fn next_states(self: &Self, piece: Self::Branch) -> Self::SelfIter;
	fn markov_state(self: &Self) -> Option<Self::MarkovState>;
}

pub trait Creatable<'a> {
	fn new(continuations: &'a HashMap<Field, HashMap<Piece, Vec<Field>>>, preview: usize, hold: bool) -> Self;
}

pub trait HasLength {
	fn len(&self) -> usize;
}

pub trait States {
	type State: StateProxy;
	fn get_state(&self, index: usize) -> Option<Self::State>;
	fn get_index(&self, state: &Self::State) -> Option<usize>;
}