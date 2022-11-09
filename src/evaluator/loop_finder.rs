use super::*;
use rayon::prelude::*;

pub struct LoopFinder {}

impl LoopFinder {
  pub fn has_loop<T: States>(mut states: ConcreteMappedStates<T>) -> bool {
    let (reversed_edge_index, reversed_edges) = {
      let mut reversed_edges = states
        .nexts
        .cont_index
        .iter()
        .enumerate()
        .flat_map(|(from, cont)| {
          let borrow_cont = &states.nexts.continuations;
          cont
            .iter()
            .cloned()
            .flat_map(move |(begin, end)| borrow_cont[begin..end].iter().map(move |&to| (to, from)))
        })
        .collect::<Vec<_>>();
      reversed_edges.par_sort_unstable();
      reversed_edges.dedup();
      let mut reversed_edge_index = vec![(0 as usize, 0 as usize); states.len()];
      let mut last = 0;
      let mut first = 0;
      for (index, (from, _)) in reversed_edges.iter().enumerate() {
        if *from != last {
          reversed_edge_index[last] = (first, index);
          first = index;
          last = *from;
        }
      }
      reversed_edge_index[last] = (first, reversed_edges.len());
      (reversed_edge_index, reversed_edges.into_iter().map(|(_, to)| to).collect::<Vec<_>>())
    };
    let mut visited = vec![false; states.len()];
    let mut stack = states.nexts.cont_index
      .par_iter()
      .enumerate()
      .filter_map(|(i, index)| index.iter().any(|&(begin, end)| begin == end).then_some(i))
      .collect::<Vec<_>>();
    for &i in &stack {
      visited[i] = true;
    }
    while let Some(i) = stack.pop() {
      let (begin, end) = reversed_edge_index[i];
      for &j in &reversed_edges[begin..end] {
        if visited[j] {
          continue;
        }
        'dead_edge: {
          let mut found = false;
          for (begin, end) in states.nexts.cont_index[j].iter_mut() {
            assert_ne!(*begin, *end);
            let begin_element = states.nexts.continuations[*begin];
            for new in states.nexts.continuations[*begin..*end].iter_mut() {
              if *new == i {
                found = true;
                *new = begin_element;
                *begin += 1;
                break;
              }
            }
            if *begin == *end {
              stack.push(j);
              visited[j] = true;
              break 'dead_edge;
            }
          }
          assert!(found);
        }
      }
    }
    visited.iter().any(|&x| !x)
  }
}
