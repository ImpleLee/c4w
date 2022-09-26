mod dashmap;
pub use self::dashmap::*;
mod recorder;
pub use recorder::*;
mod raw;
pub use raw::*;
mod parallel;
pub use parallel::*;
mod conservative;
pub use conservative::*;

use crate::states::*;
use itertools::Itertools;
use rayon::prelude::*;
use std::collections::HashMap;

pub trait Minimizer {
  fn minimize<T: States>(states: T) -> MappedStates<T>;
}
