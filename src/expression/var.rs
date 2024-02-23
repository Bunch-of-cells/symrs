use crate::Expressable;

use super::{Add, ExprKind, Expression, Mul, Tree};

#[derive(Clone, Debug, Copy, PartialEq, Eq)]
pub struct Var {
    pub(crate) id: usize,
}

impl From<Var> for Expression {
    fn from(value: Var) -> Expression {
        let mut tree = Tree::new();
        tree.push(ExprKind::Var(value));
        Expression { tree }
    }
}

impl<T> Add<T> for Var
where
    T: Expressable,
{
    type Output = Expression;
    fn add(self, rhs: T) -> Self::Output {
        let mut tree = Tree::new();
        tree.start_node(ExprKind::Add);
        tree.push_tree(Into::<Expression>::into(self).tree);
        tree.push_tree(rhs.into().tree);
        tree.finish_node();
        Expression { tree }
    }
}

impl<T> Mul<T> for Var
where
    T: Expressable,
{
    type Output = Expression;
    fn mul(self, rhs: T) -> Self::Output {
        let mut tree = Tree::new();
        tree.start_node(ExprKind::Mul);
        tree.push_tree(Into::<Expression>::into(self).tree);
        tree.push_tree(rhs.into().tree);
        tree.finish_node();
        Expression { tree }
    }
}
