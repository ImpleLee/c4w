use itertools::Itertools;
use rayon::prelude::*;

use super::Poset;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct NodeRef {
  dag: usize,
  node: usize,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Node {
  RealNode {
    id: usize
  },
  ReplacedNode {
    dag: usize
  }
}

struct DAG {
  nodes: Vec<Node>,
  edges: Vec<Vec<bool>>,
}

impl DAG {
  fn new(size: usize, edges: Vec<Vec<bool>>) -> Self {
    assert!(size > 0);
    if size == 1 {
      return Self {
        nodes: vec![Node::RealNode { id: 0 }],
        edges: vec![vec![true]],
      }
    }
    let ret = Self {
      nodes: (0..size).map(|id| Node::RealNode { id }).collect(),
      edges,
    };
    // ret.check();
    ret
  }
  fn has_relation(&self, left: usize, right: usize) -> bool {
    self.edges[left][right]
  }
  fn merge(&mut self, node: usize, replacement: Self) {
    assert!(node < self.nodes.len());
    assert!(matches!(self.nodes[node], Node::ReplacedNode { .. }));
    for (i, edge) in self.edges.iter_mut().enumerate() {
      if i == node {
        continue;
      }
      let connected = edge[node];
      edge[node] = connected;
      edge.extend(std::iter::repeat(connected).take(replacement.nodes.len() - 1));
    }
    let cloned = self.edges[node].clone();
    self.edges.extend(replacement.edges.iter().skip(1).map(|v| {
      let mut f = cloned.clone();
      f[node] = v[0];
      f.extend(v.iter().skip(1).cloned());
      f
    }));
    self.edges[node].extend(replacement.edges[0].iter().skip(1).cloned());
    self.nodes[node] = replacement.nodes[0];
    self.nodes.extend(replacement.nodes.into_iter().skip(1));
  }
  fn check(&self) {
    assert_eq!(self.nodes.len(), self.edges.len());
    for edges in self.edges.iter() {
      assert_eq!(self.nodes.len(), edges.len());
    }
    for i in 0..self.nodes.len() {
      for j in 0..self.nodes.len() {
        if i == j {
          assert!(self.edges[i][j]);
        } else {
          assert!(!self.edges[i][j] || !self.edges[j][i]);
          if !self.edges[i][j] {
            for k in 0..self.nodes.len() {
              assert!(!self.edges[i][k] || !self.edges[k][j]);
            }
          }
        }
      }
    }
  }
}

pub struct HierarchDAG {
  dags: Vec<DAG>,
  ancestors: Vec<Vec<NodeRef>>,
  id2node: Vec<NodeRef>,
}

impl HierarchDAG {
  fn calc_parents(&self, node: NodeRef) -> impl Iterator<Item=NodeRef> + '_ {
    std::iter::successors(Some(node), move |&node| {
      self.ancestors[node.dag].last().cloned()
    })
    .collect::<Vec<_>>()
    .into_iter()
    .rev()
  }
  fn rebuild_parents(&mut self) {
    for dag in 0..self.dags.len() {
      if self.ancestors[dag].is_empty() {
        continue;
      }
      self.ancestors[dag] = self.calc_parents(*self.ancestors[dag].last().unwrap()).collect();
    }
    // self.check();
  }
  fn get_parents(&self, node: NodeRef) -> impl Iterator<Item=NodeRef> + '_ {
    self.ancestors[node.dag].iter().cloned().chain(
      std::iter::once(node)
    )
  }
  fn at_mut(&mut self, node: NodeRef) -> &mut Node {
    &mut self.dags[node.dag].nodes[node.node]
  }
  fn at(&self, node: NodeRef) -> Node {
    self.dags[node.dag].nodes[node.node]
  }
  fn check(&self) {
    return;
    use itertools::zip_eq;
    for (i, &node) in self.id2node.iter().enumerate() {
      assert_eq!(self.at(node), Node::RealNode { id: i });
    }
    assert_eq!(self.dags.len(), self.ancestors.len());
    let mut roots = 0;
    for dag in 0..self.dags.len() {
      for (a, b) in zip_eq(self.calc_parents(NodeRef { dag, node: 0 }), self.get_parents(NodeRef { dag, node: 0 })) {
        assert_eq!(a, b);
      }
      let Some(&parent) = self.ancestors[dag].last() else {
        roots += 1;
        continue;
      };
      assert_eq!(self.at(parent), Node::ReplacedNode { dag });
    }
    for dag in self.dags.iter() {
      dag.check()
    }
    // assert_eq!(roots, 1);
  }
  fn merge(&mut self, dag: usize) {
    assert!(dag < self.dags.len());
    // self.check();
    let &parent = self.ancestors[dag].last().unwrap();
    let delta = self.dags[parent.dag].nodes.len() - 1;
    let mapper = |i| if i == 0 { parent.node } else { i + delta };
    for (i, &node) in self.dags[dag].nodes.iter().enumerate() {
      match node {
        Node::RealNode { id } => {
          self.id2node[id] = NodeRef { dag: parent.dag, node: mapper(i) };
        }
        Node::ReplacedNode { dag } => {
          *self.ancestors[dag].last_mut().unwrap() = NodeRef { dag: parent.dag, node: mapper(i) }
        }
      }
    }
    self.ancestors[dag] = vec![];
    let dag = std::mem::replace(&mut self.dags[dag], DAG::new(1, vec![vec![true]]));
    self.dags[parent.dag].merge(parent.node, dag);
    self.rebuild_parents();
    // self.check();
  }
}

impl Poset for HierarchDAG {
  fn new(size: usize, relations: Vec<Vec<bool>>) -> Self {
   let ret = Self {
      dags: vec![DAG::new(size, relations)],
      ancestors: vec![vec![]],
      id2node: (0..size).map(|id| NodeRef { dag: 0, node: id }).collect(),
    };
    ret.check();
    ret
  }
  fn report(&self) {
    eprintln!("poset dags: {}, nodes: {}, largest dag: {}", self.dags.len(), self.id2node.len(), self.dags.iter().map(|dag| dag.nodes.len()).max().unwrap());
  }
  fn len(&self) -> usize {
    self.id2node.len()
  }
  fn has_relation(&self, left: usize, right: usize) -> bool {
    if left == right {
      return true;
    }
    assert!(left < self.len());
    assert!(right < self.len());
    let left_parents = self.get_parents(self.id2node[left]);
    let right_parents = self.get_parents(self.id2node[right]);
    for (left, right) in left_parents.into_iter().zip(right_parents.into_iter()) {
      assert!(left.dag == right.dag);
      if left.node != right.node {
        return self.dags[left.dag].has_relation(left.node, right.node);
      }
    }
    unreachable!()
  }
  fn verify_edges(&mut self, verifier: impl Fn(&Self, usize, usize) -> bool + std::marker::Sync + std::marker::Send) -> bool {
    self.check();
    let mut checked_edges = vec![];
    for dag in self.dags.iter() {
      checked_edges.push(dag.edges.par_iter()
        .enumerate()
        .map(|(i, v)|
          v.par_iter()
            .enumerate()
            .map(|(j, &connected)| {
              if !connected {
                return false;
              }
              let Node::RealNode { id: id_i } = dag.nodes[i] else {
                return true;
              };
              let Node::RealNode { id: id_j } = dag.nodes[j] else {
                return true;
              };
              verifier(self, id_i, id_j)
            })
            .collect::<Vec<_>>()
        )
        .collect::<Vec<_>>())
    }
    let len_changed_edges = self.dags.iter_mut().zip(checked_edges.into_iter())
      .map(|(dag, mut checked_edges)| {
        std::mem::swap(&mut dag.edges, &mut checked_edges);
        checked_edges.into_iter().zip(dag.edges.iter())
          .map(|(edges, check_edges)| {
          edges.into_iter().zip(check_edges.iter())
            .filter(|&(connected, &checked)| connected != checked)
            .count()
        })
        .sum::<usize>()
      })
      .sum::<usize>();
    eprintln!("found {} internal false edges", len_changed_edges);
    len_changed_edges > 0
  }
  fn replace(&mut self, node: usize, replacement: Self) {
    assert!(node < self.len());
    self.check();
    assert_eq!(replacement.dags.len(), 1);
    let dag_delta = self.dags.len();
    let node_delta = self.id2node.len() - 1;
    let change_noderef = |noderef: NodeRef| NodeRef {
      dag: noderef.dag + dag_delta,
      node: noderef.node,
    };
    self.dags.extend(
      replacement.dags.into_iter()
        .map(|dag| DAG {
          nodes: dag.nodes.into_iter()
            .map(|new_node| match new_node {
              Node::RealNode { id } => Node::RealNode { id: if id == 0 { node } else { id + node_delta } },
              Node::ReplacedNode { dag } => Node::ReplacedNode { dag: dag + dag_delta },
            })
            .collect(),
          edges: dag.edges
        })
    );
    let parents = self.get_parents(self.id2node[node]).collect_vec();
    self.ancestors.extend(
      replacement.ancestors.into_iter()
        .map(|old_ancestors|
          parents.iter().cloned().chain(old_ancestors.into_iter().map(change_noderef)).collect()
        )
    );
    *self.at_mut(self.id2node[node]) = Node::ReplacedNode { dag: dag_delta };
    self.id2node[node] = change_noderef(replacement.id2node[0]);
    self.id2node.extend(
      replacement.id2node.into_iter()
        .skip(1)
        .map(change_noderef)
    );
    self.merge(dag_delta);
    self.dags.truncate(dag_delta);
    self.ancestors.truncate(dag_delta);
    self.check();
  }
}