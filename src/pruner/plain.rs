use crate::pruner::*;
use arrayvec::ArrayVec;
use gcd::Gcd;
use hopcroft_karp::matching;
use itertools::{iproduct, Itertools};
use rayon::prelude::*;

pub struct PlainPruner {}

impl<T: States> Pruner<T> for PlainPruner {
  fn prune(states: MappedStates<T>) -> ConcreteMappedStates<T> {
    unimplemented!()
  }
  fn prune_concrete(mut plain_states: ConcreteMappedStates<T>) -> (ConcreteMappedStates<T>, bool) {
    let states = &plain_states;
    let mut greater_than = (0..states.len())
      .into_par_iter()
      .map(|i| {
        let state = states.get_state(i).unwrap();
        let mut edges = state
          .next_pieces(states)
          .map(|piece| {
            let nexts =
              state.next_states(states, piece).map(|s| states.get_index(&s).unwrap()).collect_vec();
            iproduct!(nexts.clone().into_iter(), nexts.into_iter()).filter(|(x, y)| x < y)
          })
          .flatten()
          .collect_vec();
        edges.sort_unstable();
        edges.dedup();
        edges
      })
      .flatten()
      .filter_map(|(i, j)| find_smaller(states, i, j))
      .collect::<Vec<_>>();
    greater_than.par_sort_unstable();
    greater_than.dedup();
    greater_than.shrink_to_fit();
    eprintln!("found better edges: {}", greater_than.len());
    if greater_than.is_empty() {
      return (plain_states, false);
    }
    plain_states.nexts.cont_index.iter_mut().for_each(|x| {
      x.iter_mut().for_each(|(i, j)| {
        if *i == *j {
          return;
        }
        let nexts = &plain_states.nexts.continuations[*i..*j];
        let edges = iproduct!(nexts, nexts)
          //.filter_map(|(&x, &y)| greater_than.contains(&(x, y)).then_some(y))
          .filter_map(|(&x, &y)| greater_than.binary_search(&(x, y)).ok().map(|_| y))
          .collect_vec();
        let mut new_nexts = nexts
          .into_iter()
          .filter_map(move |x| (!edges.contains(x)).then_some(x.clone()))
          .collect_vec();
        let true_len = new_nexts.len();
        if *j - *i == true_len {
          return;
        }
        new_nexts.extend((0..(*j - *i - true_len)).map(|_| usize::MAX));
        plain_states.nexts.continuations[*i..*j].clone_from_slice(&new_nexts);
        *j = *i + true_len;
      })
    });
    plain_states.nexts.continuations.retain(|&x| x != usize::MAX);
    plain_states.nexts.continuations.shrink_to_fit();
    let mut last = 0;
    for x in plain_states.nexts.cont_index.iter_mut() {
      for (i, j) in x.iter_mut() {
        let true_len = *j - *i;
        if *i != last {
          *i = last;
          *j = last + true_len;
        }
        last = *j;
      }
    }
    assert_eq!(last, plain_states.nexts.continuations.len());
    (plain_states, true)
  }
}

fn find_smaller<T: States>(states: &T, u1: usize, u2: usize) -> Option<(usize, usize)> {
  if u1 == u2 {
    return Some((u1, u2));
  }
  let get_nexts = |u: usize| -> ArrayVec<Vec<usize>, 7> {
    let s = states.get_state(u).unwrap();
    let mut nexts: ArrayVec<Vec<usize>, 7> = s
      .next_pieces(states)
      .map(|piece| {
        s.next_states(states, piece).map(|state| states.get_index(&state.gen()).unwrap()).collect()
      })
      .collect();
    nexts.sort_unstable_by_key(|v| v.len());
    nexts
  };
  let nexts1 = get_nexts(u1);
  let nexts2 = get_nexts(u2);
  if nexts1.len() == 0 {
    return Some((u2, u1));
  } else if nexts2.len() == 0 {
    return Some((u1, u2));
  }

  let get_multiplier = |a: usize, b: usize| (b / a.gcd(b), a / a.gcd(b));
  let (multiplier1, multiplier2) = get_multiplier(nexts1.len(), nexts2.len());
  let bases = iproduct!(
    (0..multiplier1).map(|i| i * nexts1.len()),
    (multiplier2..multiplier2 * 2).map(|i| i * nexts2.len())
  );
  let edging_size = nexts1.len() * multiplier1;
  let edge_1_2 = iproduct!(nexts1.iter().enumerate(), nexts2.iter().enumerate())
    .filter_map(|((i1, next1), (i2, next2))| {
      (next1.len() <= next2.len() && next1.iter().all(|i| next2.contains(i)))
        .then_some(bases.clone().map(move |(b1, b2)| (b1 + i1, b2 + i2)))
    })
    .flatten()
    .collect_vec();
  if matching(&edge_1_2).len() >= edging_size {
    return Some((u2, u1));
  }
  let edge_2_1 = iproduct!(nexts1.iter().enumerate(), nexts2.iter().enumerate())
    .filter_map(|((i1, next1), (i2, next2))| {
      (next2.len() <= next1.len() && next2.iter().all(|i| next1.contains(i)))
        .then_some(bases.clone().map(move |(b1, b2)| (b1 + i1, b2 + i2)))
    })
    .flatten()
    .collect_vec();
  if matching(&edge_2_1).len() >= edging_size {
    return Some((u1, u2));
  }
  None
}
