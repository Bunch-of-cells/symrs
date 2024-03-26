use super::{ExprKind, Expressable, Expressand, Expression, Tree};

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct Var {
    pub(crate) id: usize,
}

impl From<Var> for Expression {
    fn from(value: Var) -> Expression {
        let mut tree = Tree::new();
        tree.push(ExprKind::Var(value));
        Expressable(Expressand { tree })
    }
}
