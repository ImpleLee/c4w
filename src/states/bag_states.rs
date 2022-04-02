// #[derive(Debug, Hash, Eq, PartialEq, Serialize, Deserialize, Copy, Clone, Ord, PartialOrd)]
// enum Bag {
//   TwoBags(usize),
//   // one bag will never be all true
//   OneBag([bool; 7]),
// }

// #[derive(Debug, Clone, PartialEq, Eq, Hash, Ord, PartialOrd)]
// struct BagState {
//   field: Field,
//   hold: Option<Piece>,
//   preview: VecDeque<Piece>,
//   bag: Bag,
// }

// impl BagState {
//   fn push(&self, piece: Piece) -> Self {
//     let mut ret = self.clone();
//     ret.preview.push_back(piece);
//     match ret.bag {
//       Bag::TwoBags(_) => {},
//       Bag::OneBag(ref mut bag) => {
//         bag[piece as usize] = true;
//         if bag.iter().all(|&b| b) {
//           ret.bag = Bag::TwoBags(ret.preview.len());
//         }
//       }
//     }
//     ret
//   }
//   fn pull(self: &Self) -> (Self, Piece) {
//     let mut ret = self.clone();
//     let piece = ret.preview.pop_front().unwrap();
//     match ret.bag {
//       Bag::TwoBags(n) => {
//         if n == 1 {
//           ret.bag = Bag::OneBag([false; 7]);
//           match ret.bag {
//             Bag::OneBag(ref mut bag) => {
//               for i in ret.preview.iter() {
//                 bag[*i as usize] = true;
//               }
//             }
//             _ => unreachable!()
//           }
//         } else {
//           ret.bag = Bag::TwoBags(n - 1);
//         }
//       },
//       Bag::OneBag(_) => {}
//     }
//     (ret, piece)
//   }
// }

// impl State for BagState {
//   fn new(fields: Vec<&Field>, preview: usize, hold: bool) -> Vec<Self> {
//     assert!(preview <= 13);
//     let mut res = vec![BagState{ field: Field([0; 4]), preview: VecDeque::new(), hold: None, bag: Bag::OneBag([false; 7]) }];
//     for _ in 0..preview {
//       let mut temp = vec![];
//       for state in res {
//         for piece in state.next_pieces() {
//           temp.push(state.push(piece));
//         }
//       }
//       res = temp;
//     }
//     let mut res = res.iter().map(|s| s.clone()).collect::<HashSet<_>>();
//     let mut q = VecDeque::new();
//     for state in res.iter() {
//       q.push_back(state.clone());
//     }
//     while q.len() > 0 {
//       let state = q.pop_front().unwrap();
//       for piece in state.next_pieces() {
//         let (new_state, _) = state.push(piece).pull();
//         if res.contains(&new_state) {
//           continue;
//         }
//         res.insert(new_state.clone());
//         q.push_back(new_state);
//       }
//     }
//     println!("{}", res.len());
//     let mut res = res.iter().map(|s| fields.iter().map(|f| s.change_field(**f)).collect::<Vec<_>>()).flatten().collect::<Vec<_>>();
//     if hold {
//       let mut temp = vec![];
//       for state in res {
//         for piece in PIECES {
//           temp.push(BagState{ hold: Some(piece), ..state.clone() });
//         }
//       }
//       res = temp;
//     }
//     res
//   }

//   fn next_pieces(self: &Self) -> Vec<Piece> {
//     match self.bag {
//       Bag::TwoBags(n) => {
//         let mut temp = PIECES.iter().map(|&p| p).collect::<HashSet<Piece>>();
//         for p in n..self.preview.len() {
//           temp.remove(&self.preview[p]);
//         }
//         temp.into_iter().collect::<Vec<_>>()
//       }
//       Bag::OneBag(ref bag) => {
//         let mut res = vec![];
//         for i in 0..7 {
//           if !bag[i] {
//             res.push(Piece::num2piece(i));
//           }
//         }
//         res
//       }
//     }
//   }

//   fn change_field(&self, field: Field) -> Self {
//     BagState{ field, ..self.clone() }
//   }

//   fn add_piece(&self, piece: Piece) -> Vec<(Self, Piece)> {
//     let mut vec = vec![];
//     let ret = self.push(piece);
//     let (ret, p) = ret.pull();
//     if let Some(hold) = self.hold {
//       if hold != p {
//         let mut ret = ret.clone();
//         ret.hold = Some(p);
//         vec.push((ret, hold));
//       }
//     }
//     vec.push((ret, p));
//     vec
//   }

//   fn get_field(self: &Self) -> Field {
//     self.field
//   }
// }