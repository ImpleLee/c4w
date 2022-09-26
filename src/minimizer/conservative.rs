use crate::minimizer::*;

pub struct ConservMinimizer {}

impl Minimizer for ConservMinimizer {
  fn minimize<T: States>(states: T) -> MappedStates<T> {
    let mut state_sorted = (0..states.len()).collect_vec();
    eprintln!("start sorting");
    state_sorted.par_sort_unstable_by_key(|&i| states.get_next_id(i, None));
    eprintln!("finish sorting");
    let state_order = state_sorted.clone();
    let (_, _, inverse) = state_sorted.iter_mut().fold(
      (0, states.get_next_id(state_order[0], None), vec![state_order[0]]),
      |(mut num, v, mut indices), i| {
        let next = states.get_next_id(*i, None);
        if next != v {
          num += 1;
          indices.push(*i);
        }
        *i = num;
        (num, next, indices)
      }
    );
    let mut state_order = state_order
      .par_iter()
      .zip(state_sorted.par_iter())
      .map(|(&i, &j)| (i, j))
      .collect::<Vec<_>>();
    state_order.par_sort_unstable_by_key(|&(i, _)| i);
    let mapping = state_order.par_iter().map(|&(_, j)| j).collect::<Vec<_>>();
    MappedStates { original: states, mapping, inverse }
  }
}
