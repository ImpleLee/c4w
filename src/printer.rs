use crate::states::States;





pub trait Printer {
  fn print<T: States>(field2state: &[usize], values: &[f64], num2state: &T);
}

// pub struct MarkovAverage();

// impl Printer for MarkovAverage {
//   fn print<T: States>(field2state: &Vec<usize>, values: &Vec<f64>, num2state: T) {
//     let mut states = field2state.iter().enumerate().filter_map(|(i, field)| {
//       match (&num2state).get_state(i).unwrap().markov_state() {
//         Some(state) => Some((state, values[*field])),
//         None => None
//       }
//     }).collect_vec();
//     states.par_sort_unstable_by_key(|(state, _)| state.clone());
//     let mut values =
//       states.into_iter()
//       .group_by(|(state, _)| state.clone())
//       .into_iter()
//       .map(|(f, states)| (f, states.map(|(_, v)| v).collect::<Mean>().mean()))
//       .collect_vec();
//     values.par_sort_unstable_by_key(|&(_, value)| NotNan::new(-value).unwrap());
//     for (field, value) in values.iter() {
//       println!("{}", value);
//       println!("{}", field);
//     }
//   }
// }
