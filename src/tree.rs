use crate::ExprKind;

pub type Span = std::ops::Range<usize>;

#[derive(Clone, Debug)]
pub struct Tree {
    nodes: Vec<Node>,
    current: Vec<usize>,
}

impl Tree {
    pub fn new() -> Tree {
        let root = Node {
            kind: ExprKind::ROOT,
            parent: NodeId(0),
            children: Vec::new(),
            id: 0,
        };
        Self {
            nodes: vec![root],
            current: Vec::new(),
        }
    }

    pub fn root(&self) -> &Node {
        &self.nodes[0]
    }

    pub fn sub_roots(&self) -> Vec<&Node> {
        self.root().children.iter().map(|&i| self.node(i)).collect()
    }

    #[track_caller]
    pub fn node(&self, id: NodeId) -> &Node {
        &self.nodes[id.0]
    }

    #[track_caller]
    pub fn node_mut(&mut self, id: NodeId) -> &mut Node {
        &mut self.nodes[id.0]
    }

    pub fn start_node(&mut self, kind: ExprKind) {
        let current = self.nodes.len();
        let parent = self.current.last().copied().unwrap_or(0);
        self.current.push(current);
        let current = NodeId(current);

        self.nodes[parent].children.push(current);
        self.nodes.push(Node {
            kind,
            parent: NodeId(parent),
            children: Vec::new(),
            id: current.0,
        });
    }

    pub fn push_tree(&mut self, mut tree: Tree) {
        assert!(tree.current.is_empty());
        let parent = NodeId(self.current.last().copied().unwrap_or(0));

        let children = tree.nodes.remove(0).children;

        let len = self.nodes.len() - 1;
        for node in &mut tree.nodes {
            node.id += len;
            for child in &mut node.children {
                child.0 += len;
            }
        }
        self.nodes.append(&mut tree.nodes);

        for child in children {
            self.nodes[parent.0].children.push(NodeId(child.0 + len));
            self.nodes[child.0 + len].parent = parent;
        }
    }

    pub fn push(&mut self, kind: ExprKind) -> NodeId {
        self.start_node(kind);
        self.finish_node()
    }

    #[track_caller]
    pub fn finish_node(&mut self) -> NodeId {
        let current = self.current.pop().expect("No node to finish");
        NodeId(current)
    }
}

impl Default for Tree {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug)]
pub struct Node {
    pub(crate) kind: ExprKind,
    pub(crate) parent: NodeId,
    pub(crate) children: Vec<NodeId>,
    pub(crate) id: usize,
}

impl Node {
    /// Type of the node
    pub fn kind(&self) -> ExprKind {
        self.kind
    }

    /// id of the node
    pub fn id(&self) -> NodeId {
        NodeId(self.id)
    }

    /// Parent of the node
    pub fn parent(&self) -> NodeId {
        self.parent
    }

    /// Children nodes
    pub fn children(&self) -> &Vec<NodeId> {
        &self.children
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct NodeId(pub(crate) usize);

impl NodeId {
    pub const ROOT: Self = NodeId(0);
}
