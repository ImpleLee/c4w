use crate::minimizer::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::iter::Iterator;

pub struct ParallelMinimizer {}

impl Minimizer for ParallelMinimizer {
  fn minimize<T: States+std::marker::Sync+HasLength>(states: T) -> MappedStates<T> {
    let mut mapping = vec![0_usize; states.len()];
    mapping.shrink_to_fit();
    let mut inverse = vec![0];
    loop {
      let global = inverse
        .par_iter()
        .map(|&state| (states.get_next_id(state, &*mapping), state))
        .collect::<HashMap<_, _>>();
      let mut new_mapping = vec![0_usize; states.len()];
      new_mapping.shrink_to_fit();
      let local_seeds = new_mapping
        .par_iter_mut()
        .enumerate()
        .fold(HashMap::new, |mut local, (i, ret)| {
          let next = states.get_next_id(i, &*mapping);
          *ret = match global.get(&next) {
            Some(&known) => known,
            None => *local.entry(next).or_insert(i)
          };
          local
        })
        .map(|local| local.into_iter().map(|(k, v)| (k, vec![v])).collect())
        .reduce(HashMap::new, |mut local1, local2| {
          for (next, mut seeds) in local2 {
            local1.entry(next).or_default().append(&mut seeds)
          }
          local1
        })
        .into_values()
        .collect::<Vec<_>>();
      mapping = new_mapping;
      inverse.extend(local_seeds.iter().map(|seeds| seeds[0]));
      eprintln!("minimized states: {}", inverse.len());
      if local_seeds.len() == 0 {
        break;
      }
      let seed_dedup = local_seeds
        .into_par_iter()
        .flat_map(|seeds| {
          let seed0 = seeds[0];
          seeds.into_par_iter().skip(1).map(move |seed| (seed, seed0))
        })
        .collect::<HashMap<_, _>>();
      eprintln!("need to dedup: {}", seed_dedup.len());
      if !seed_dedup.is_empty() {
        mapping.par_iter_mut().for_each(|res| {
          if let Some(&seed) = seed_dedup.get(res) {
            *res = seed;
          }
        });
      }
    }
    let seed_map =
      inverse.par_iter().enumerate().map(|(i, &seed)| (seed, i)).collect::<HashMap<_, _>>();
    mapping.par_iter_mut().for_each(|res| {
      *res = seed_map[res];
    });
    MappedStates { original: states, mapping, inverse }
  }
}
