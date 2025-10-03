use arrayvec::ArrayVec;
use rs_graph::{maxflow::PushRelabel, vecgraph::VecGraphBuilder, Builder};

use crate::states::*;

mod posets;
pub use posets::*;
mod provers;
pub use provers::*;

#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone)]
struct Branch(Vec<usize>);
impl Branch {
  fn is_geq(&self, other: &Self, geq: impl Fn(usize, usize) -> bool) -> bool {
    // max(left) >= max(right) <=> forall i in right, exists j in left, j >= i
    other.0.iter().all(|&r| self.0.iter().any(|&l| geq(l, r)))
  }
}

#[derive(Clone)]
struct Next(ArrayVec<usize, 7>);
impl Next {
  fn is_geq(&self, other: &Self, branch_geq: impl Fn(usize, usize) -> bool) -> bool {
    let left = self.0.clone();
    let right = other.0.clone();
    let mut graph_builder = VecGraphBuilder::<usize>::with_capacities(
      2 + left.len() + right.len(),
      (1 + left.len()) * (1 + right.len()) - 1
    );
    let left_node = graph_builder.add_node();
    let left_branches = graph_builder.add_nodes(left.len());
    use rs_graph::traits::Indexable;
    for &left_branch in left_branches.iter() {
      let edge = graph_builder.add_edge(left_node, left_branch);
      assert!(edge.index() < left.len());
    }
    let right_node = graph_builder.add_node();
    let right_branches = graph_builder.add_nodes(right.len());
    for &right_branch in right_branches.iter() {
      let edge = graph_builder.add_edge(right_branch, right_node);
      assert!(edge.index() >= left.len() && edge.index() < left.len() + right.len());
    }
    for (&left_id, &left_branch) in left.iter().zip(left_branches.iter()) {
      for (&right_id, &right_branch) in right.iter().zip(right_branches.iter()) {
        if branch_geq(left_id, right_id) {
          graph_builder.add_edge(left_branch, right_branch);
        }
      }
    }
    let graph = graph_builder.into_graph();
    let mut push_relabel = PushRelabel::new(&graph);
    push_relabel.solve(left_node, right_node, |e| {
      if e.index() < left.len() {
        right.len()
      } else if e.index() < left.len() + right.len() {
        left.len()
      } else {
        usize::MAX
      }
    });
    push_relabel.value() == left.len() * right.len()
  }
}

pub trait ProvePruner {
  fn prune<T: States>(states: ConcreteMappedStates<T>) -> ConcreteMappedStates<T>;
}
