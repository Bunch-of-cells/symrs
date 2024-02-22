use super::*;

#[derive(Clone, Debug)]
pub struct Leaf {
    pub(crate) kind: ExprKind,
    pub(crate) id: usize,
}

impl Leaf {
    /// Type of the leaf
    pub fn kind(&self) -> ExprKind {
        self.kind.clone()
    }

    /// id of the leaf
    pub fn id(&self) -> LeafId {
        LeafId(self.id)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct LeafId(pub(crate) usize);

impl LeafId {
    /// Get leaf from tree
    pub fn get(self, tree: &Tree) -> &Leaf {
        &tree.leaves[self.0]
    }

    /// Get leaf from tree builder
    pub fn get_from_builder(self, tree: &Tree) -> &Leaf {
        &tree.leaves[self.0]
    }
}
