use super::*;

#[derive(Clone, Debug)]
pub struct Node {
    pub(crate) kind: ExprKind,
    pub(crate) leaf_span: Span,
    pub(crate) parent: usize,
    pub(crate) children: Vec<usize>,
    pub(crate) prev: Option<usize>,
    pub(crate) next: Option<usize>,
    pub(crate) id: usize,
}

impl Node {
    /// Type of the node
    pub fn kind(&self) -> ExprKind {
        self.kind.clone()
    }

    /// id of the node
    pub fn id(&self) -> NodeId {
        NodeId(self.id)
    }

    /// Parent of the node
    pub fn parent(&self) -> NodeId {
        NodeId(self.parent)
    }

    /// Next sibling node
    pub fn next(&self) -> Option<NodeId> {
        self.next.map(NodeId)
    }

    /// Previous sibling node
    pub fn prev(&self) -> Option<NodeId> {
        self.prev.map(NodeId)
    }

    /// Children nodes
    pub fn children(&self) -> &Vec<NodeId> {
        unsafe { std::mem::transmute(&self.children) }
    }

    /// Children nodes, including leaves
    pub fn children_with_leaves<'a>(&self, tree: &'a Tree) -> ChildLeafIter<'a> {
        ChildLeafIter {
            tree,
            leaf: self.leaf_span.start,
            child: 0,
            node: self.id,
        }
    }

    /// Children leaves
    pub fn leaves<'a, L>(&self, tree: &'a Tree) -> &'a [Leaf] {
        &tree.leaves[self.leaf_span.clone()]
    }

    pub fn iter_bfs<'a, L>(&self, tree: &'a Tree) -> TreeIterBfs<'a> {
        TreeIterBfs {
            tree,
            queue: VecDeque::from([TreeElement::Node(NodeId(self.id))]),
        }
    }

    pub fn iter_dfs<'a, L>(&self, tree: &'a Tree) -> TreeIterDfs<'a> {
        TreeIterDfs {
            tree,
            stack: Vec::from([TreeElement::Node(NodeId(self.id))]),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct NodeId(pub(crate) usize);

impl NodeId {
    pub const ROOT: Self = NodeId(0);

    /// Get node from tree
    pub fn get(self, tree: &Tree) -> &Node {
        &tree.nodes[self.0]
    }

    /// Get node from tree builder
    pub fn get_from_builder(self, tree: &Tree) -> &Node {
        &tree.nodes[self.0]
    }
}
