use std::collections::HashSet;

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
  fn get_topo_order(size: usize, relation: impl Fn(usize, usize) -> bool + std::marker::Sync) -> Vec<usize> {
    let mut result = (0..size).collect::<Vec<_>>();
    result.par_sort_by_cached_key(|&id| std::cmp::Reverse((0..size).filter(|&other| relation(id, other)).count()));
    result
  }
  fn get_rev_reduction(topo_order: Vec<usize>, relation: impl Fn(usize, usize) -> bool) -> impl Iterator<Item=(usize, Vec<usize>)> {
    let rev_topo_order = {
      let mut result = vec![0; topo_order.len()];
      for (i, &id) in topo_order.iter().enumerate() {
        result[id] = i;
      }
      result
    };
    let mut result = vec![vec![]; topo_order.len()];
    let mut marks = vec![false; topo_order.len()];
    (1..topo_order.len()).map(move |i| {
      for j in 0..i {
        marks[j] = false;
      }
      for j in (0..i).rev() {
        if relation(topo_order[j], topo_order[i]) {
          if !marks[j] {
            result[topo_order[i]].push(topo_order[j]);
            marks[j] = true;
          }
        }
        if marks[j] {
          for &k in result[topo_order[j]].iter() {
            marks[rev_topo_order[k]] = true;
          }
        }
      }
      (topo_order[i], result[topo_order[i]].clone())
    })
  }
  fn has_relation(&self, left: usize, right: usize) -> bool {
    self.edges[left][right]
  }
  fn header(&self) -> impl Iterator<Item=usize> + '_ {
    (0..self.nodes.len()).filter(move |&i| self.edges.iter().enumerate().all(|(j, v)| i == j || !v[i]))
  }
  fn footer(&self) -> impl Iterator<Item=usize> + '_ {
    self.edges.iter().enumerate().filter(move |&(i, v)| v.iter().enumerate().all(|(j, &e)| i == j || !e)).map(|(i, _)| i)
  }
  fn remove_edge(&mut self, left: usize, right: usize) {
    assert!(self.edges[left][right]);
    self.edges[left][right] = false;
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
  fn for_direct_outs(&self, node: NodeRef, mut merging_stack: &mut Vec<usize>, f: &mut impl FnMut(usize, &mut Vec<usize>)) {
    match self.at(node) {
      Node::RealNode { id } => {
        f(id, merging_stack);
      }
      Node::ReplacedNode { dag } => {
        merging_stack.push(dag);
        for i in self.dags[dag].footer() {
          self.for_direct_outs(NodeRef { dag, node: i }, &mut merging_stack, f);
        }
        merging_stack.pop();
      }
    }
  }
  fn for_direct_ins(&self, node: NodeRef, mut merging_stack: &mut Vec<usize>, f: &mut impl FnMut(usize, &mut Vec<usize>)) {
    match self.at(node) {
      Node::RealNode { id } => {
        f(id, merging_stack);
      }
      Node::ReplacedNode { dag } => {
        merging_stack.push(dag);
        for i in self.dags[dag].header() {
          self.for_direct_ins(NodeRef { dag, node: i }, &mut merging_stack, f);
        }
        merging_stack.pop();
      }
    }
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
    let mut false_edges = vec![];
    for dag in self.dags.iter() {
      let topo_order = DAG::get_topo_order(dag.nodes.len(), |i, j| dag.edges[i][j]);
      let false_edge = DAG::get_rev_reduction(topo_order, |i, j| dag.edges[i][j]).par_bridge()
        .flat_map(|(j, v)| v.into_par_iter().map(move |i| (i, j)))
        .filter(|&(i, j)| {
          let Node::RealNode { id: id_i } = dag.nodes[i] else {
            return false;
          };
          let Node::RealNode { id: id_j } = dag.nodes[j] else {
            return false;
          };
          !verifier(self, id_i, id_j)
        })
        .collect::<Vec<_>>();
      false_edges.push(false_edge);
    }
    let len_false_edges = false_edges.iter().map(|v| v.len()).sum::<usize>();
    eprintln!("found {} internal false edges", len_false_edges);
    if len_false_edges > 0 {
      for (d, v) in false_edges.into_iter().enumerate() {
        for (i, j) in v {
          self.dags[d].remove_edge(i, j);
        }
      }
      self.check();
      return true;
    }
    let mut false_edges = vec![];
    let mut merged_dags = HashSet::new();
    for (d, dag) in self.dags.iter().enumerate() {
      let topo_order = DAG::get_topo_order(dag.nodes.len(), |i, j| dag.edges[i][j]);
      for (i, j) in DAG::get_rev_reduction(topo_order, |i, j| dag.edges[i][j])
        .into_iter()
        .flat_map(|(j, v)| v.into_iter().map(move |i| (i, j))) {
        if matches!((dag.nodes[i], dag.nodes[j]), (Node::RealNode { .. }, Node::RealNode { .. })) {
          continue;
        }
        let mut merging_stack = Vec::with_capacity(self.dags.len());
        self.for_direct_outs(NodeRef{ dag: d, node: i }, &mut merging_stack, &mut |i, mut merging_stack| {
          self.for_direct_ins(NodeRef { dag: d, node: j }, &mut merging_stack, &mut |j, merging_stack| {
            if !verifier(self, i, j) {
              false_edges.push((i, j));
              merged_dags.extend(merging_stack.iter().cloned())
            }
          })
        });
        assert!(merging_stack.is_empty());
      }
    }
    if false_edges.is_empty() {
      return false;
    }
    eprintln!("found {} cross-dag false edges", false_edges.len());
    eprintln!("have to merge {} dags", merged_dags.len());
    let mut merged_dags = merged_dags.into_iter().collect_vec();
    merged_dags.sort();
    for dag in merged_dags.into_iter().rev() {
      self.merge(dag);
    }
    self.check();
    for (i, j) in false_edges {
      assert_eq!(self.id2node[i].dag, self.id2node[j].dag);
      let dag = self.id2node[i].dag;
      self.dags[dag].remove_edge(self.id2node[i].node, self.id2node[j].node);
    }
    self.check();
    true
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
    self.check();
  }
}