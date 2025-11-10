mod dashmap;

mod recorder;

mod raw;

mod parallel;
pub use parallel::*;
mod conservative;


use crate::states::*;
use itertools::Itertools;
use rayon::prelude::*;
use std::collections::HashMap;

pub trait Minimizer {
  fn minimize<T: States>(states: T) -> MappedStates<T>;
}
