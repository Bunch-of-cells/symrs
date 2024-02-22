mod leaf;
mod node;
pub use leaf::*;
pub use node::*;

use crate::ExprKind;
use std::collections::VecDeque;

pub type Span = std::ops::Range<usize>;

#[derive(Clone, Debug)]
pub struct Tree {
    leaves: Vec<Leaf>,
    nodes: Vec<Node>,
    current: Vec<usize>,
}

impl Tree {
    pub fn new() -> Tree {
        let node = Node {
            kind: ExprKind::Root,
            leaf_span: Span { start: 0, end: 0 },
            parent: 0,
            children: Vec::new(),
            prev: None,
            next: None,
            id: 0,
        };
        Tree {
            leaves: Vec::new(),
            nodes: vec![node],
            current: Vec::new(),
        }
    }

    pub fn leaf(&self, id: LeafId) -> &Leaf {
        &self.leaves[id.0]
    }

    pub fn leaf_mut(&mut self, id: LeafId) -> &mut Leaf {
        &mut self.leaves[id.0]
    }

    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id.0]
    }

    pub fn node_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes[id.0]
    }

    pub fn start_node(&mut self, kind: ExprKind) {
        let current = self.nodes.len();
        let parent = self.current.last().copied().unwrap_or(0);
        self.current.push(current);
        let mut prev = None;
        if let Some(p) = self.nodes[parent].children.last().copied() {
            prev = Some(p);
            self.nodes[p].next = Some(current);
        }

        self.nodes[parent].children.push(current);
        self.nodes.push(Node {
            kind,
            parent,
            leaf_span: self.leaves.len()..0,
            children: Vec::new(),
            prev,
            next: None,
            id: current,
        });
    }

    pub fn push(&mut self, kind: ExprKind) -> LeafId {
        let current = self.leaves.len();
        let id = LeafId(current);
        self.leaves.push(Leaf { kind, id: current });
        self.nodes[0].leaf_span.end = self.leaves.len();
        id
    }

    pub fn push_tree(&mut self, mut tree: Tree) -> NodeId {
        let current = self.nodes.len();
        let parent = self.current.last().copied().unwrap_or(0);
        let mut prev = None;
        if let Some(p) = self.nodes[parent].children.last().copied() {
            prev = Some(p);
            self.nodes[p].next = Some(current);
        }

        self.nodes[parent].children.push(current);
        tree.nodes[0].parent = parent;
        tree.nodes[0].prev = prev;
        for node in &mut tree.nodes {
            node.leaf_span.start += self.leaves.len();
            node.leaf_span.end += self.leaves.len();
            node.id += self.nodes.len();
            for child in &mut node.children {
                *child += self.nodes.len();
            }
        }
        for leaf in &mut tree.leaves {
            leaf.id += self.leaves.len();
        }
        self.leaves.append(&mut tree.leaves);
        self.nodes.append(&mut tree.nodes);
        self.nodes[0].leaf_span.end = self.leaves.len();
        NodeId(current)
    }

    #[track_caller]
    pub fn finish_node(&mut self) -> NodeId {
        let current = self.current.pop().expect("No node to finish");
        let id = NodeId(current);
        self.nodes[current].leaf_span.end = self.leaves.len();
        id
    }

    pub fn checkpoint(&self) -> Checkpoint {
        Checkpoint {
            child_no: self.nodes[self.current.last().copied().unwrap_or(0)]
                .children
                .len(),
            leaf: self.leaves.len(),
            parent: self.current.last().copied().unwrap_or(0),
        }
    }

    pub fn start_node_at(&mut self, checkpoint: Checkpoint, kind: ExprKind) {
        let Checkpoint {
            child_no,
            leaf,
            parent,
        } = checkpoint;
        let current = self.nodes.len();
        self.current.push(current);

        let mut prev = None;
        let mut children = Vec::new();

        let nodes = self.nodes[parent]
            .children
            .drain(child_no..)
            .collect::<Vec<_>>();
        let mut nodes = nodes.iter();

        if let Some(&first) = nodes.next() {
            self.nodes[first].prev = None;
            self.nodes[first].parent = current;
            children.push(first);
        }

        for &node in nodes {
            self.nodes[node].parent = current;
            children.push(node);
        }

        if child_no != 0 {
            if let Some(&p) = self.nodes[parent].children.get(child_no - 1) {
                prev = Some(p);
                self.nodes[p].next = Some(current);
            }
        }

        self.nodes[parent].children.push(current);
        self.nodes.push(Node {
            kind,
            parent,
            leaf_span: leaf..0,
            children,
            prev,
            next: None,
            id: current,
        });
    }

    pub fn iter_bfs(&self) -> TreeIterBfs<'_> {
        self.do_root_node_stuff();
        TreeIterBfs {
            tree: self,
            queue: VecDeque::from([TreeElement::Node(NodeId(0))]),
        }
    }

    pub fn iter_dfs(&self) -> TreeIterDfs<'_> {
        self.do_root_node_stuff();
        TreeIterDfs {
            tree: self,
            stack: Vec::from([TreeElement::Node(NodeId(0))]),
        }
    }

    fn do_root_node_stuff(&self) {
        assert!(self.current.is_empty());
    }
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum TreeElement {
    Node(NodeId),
    Leaf(LeafId),
}

impl TreeElement {
    /// Converts into leaf, if it is a leaf
    pub fn into_leaf(self) -> Option<LeafId> {
        if let Self::Leaf(l) = self {
            Some(l)
        } else {
            None
        }
    }

    /// Converts into node, if it is a node
    pub fn into_node(self) -> Option<NodeId> {
        if let Self::Node(n) = self {
            Some(n)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone)]
pub struct Checkpoint {
    child_no: usize,
    leaf: usize,
    parent: usize,
}

pub struct TreeIterBfs<'a> {
    tree: &'a Tree,
    queue: VecDeque<TreeElement>,
}

impl<'a> Iterator for TreeIterBfs<'a> {
    type Item = TreeElement;
    fn next(&mut self) -> Option<Self::Item> {
        let node = match self.queue.pop_front()? {
            TreeElement::Node(NodeId(n)) => &self.tree.nodes[n],
            n @ TreeElement::Leaf(_) => return Some(n),
        };
        self.queue.extend(node.children_with_leaves(self.tree));
        Some(TreeElement::Node(NodeId(node.id)))
    }
}

pub struct TreeIterDfs<'a> {
    tree: &'a Tree,
    stack: Vec<TreeElement>,
}

impl<'a> Iterator for TreeIterDfs<'a> {
    type Item = TreeElement;
    fn next(&mut self) -> Option<Self::Item> {
        let node = match self.stack.pop()? {
            TreeElement::Node(NodeId(n)) => &self.tree.nodes[n],
            n @ TreeElement::Leaf(_) => return Some(n),
        };
        self.stack.extend(node.children_with_leaves(self.tree));
        Some(TreeElement::Node(NodeId(node.id)))
    }
}

pub struct ChildLeafIter<'a> {
    tree: &'a Tree,
    leaf: usize,
    child: usize,
    node: usize,
}

impl<'a> Iterator for ChildLeafIter<'a> {
    type Item = TreeElement;
    fn next(&mut self) -> Option<Self::Item> {
        let current = &self.tree.nodes[self.node];
        if self.leaf >= current.leaf_span.end {
            return None;
        }
        let child_id = current.children.get(self.child).copied();
        let child_span = child_id.map(|id| self.tree.nodes[id].leaf_span.clone());
        match (child_id, child_span) {
            (Some(id), Some(Span { start, end })) if start == self.leaf => {
                self.leaf = end;
                self.child += 1;
                Some(TreeElement::Node(NodeId(id)))
            }
            _ => {
                let out = self.leaf;
                self.leaf += 1;
                Some(TreeElement::Leaf(LeafId(out)))
            }
        }
    }
}
