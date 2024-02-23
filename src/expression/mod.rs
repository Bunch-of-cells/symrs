pub mod var;
use crate::tree::*;
pub use var::*;

use std::ops::{Add, Mul};

#[derive(Debug, Clone)]
pub struct Expression {
    pub(crate) tree: Tree,
}

impl Expression {
    pub fn diff(&self, x: Var) -> Expression {
        Expression {
            tree: self.d_inner(NodeId::ROOT, x).unwrap_or_default(),
        }
    }

    fn d_inner(&self, id: NodeId, x: Var) -> Option<Tree> {
        match self.tree.node(id).kind {
            ExprKind::ROOT => {
                let mut trees = self
                    .tree
                    .node(id)
                    .children()
                    .iter()
                    .filter_map(|&id| self.d_inner(id, x))
                    .collect::<Vec<_>>();

                match trees[..] {
                    [_] | [] => trees.pop(),
                    _ => {
                        let mut tree = Tree::new();
                        for tre in trees {
                            tree.push_tree(tre);
                        }
                        Some(tree)
                    }
                }
            }
            ExprKind::Add => {
                let mut trees = self
                    .tree
                    .node(id)
                    .children()
                    .iter()
                    .filter_map(|&id| self.d_inner(id, x))
                    .collect::<Vec<_>>();

                match trees[..] {
                    [_] | [] => trees.pop(),
                    _ => {
                        let mut tree = Tree::new();
                        tree.start_node(ExprKind::Add);
                        for tre in trees {
                            tree.push_tree(tre);
                        }
                        tree.finish_node();
                        Some(tree)
                    }
                }
            }
            ExprKind::Mul => {
                let children = self.tree.node(id).children();
                let mut trees = children
                    .iter()
                    .enumerate()
                    .filter_map(|(i, &id)| self.d_inner(id, x).map(|d| (i, d)))
                    .map(|(i, d)| {
                        let mut new_tree = Tree::new();
                        new_tree.start_node(ExprKind::Mul);
                        new_tree.push_tree(d);
                        for (j, child) in children.iter().enumerate() {
                            if i == j {
                                continue;
                            }
                            new_tree.push_tree(self.treeify_node(*child));
                        }
                        new_tree.finish_node();
                        new_tree
                    })
                    .collect::<Vec<_>>();

                match trees[..] {
                    [_] | [] => trees.pop(),
                    _ => {
                        let mut tree = Tree::new();
                        tree.start_node(ExprKind::Add);
                        for tre in trees {
                            tree.push_tree(tre);
                        }
                        tree.finish_node();
                        Some(tree)
                    }
                }
            }
            ExprKind::Var(v) if v.id == x.id => {
                let mut new_tree = Tree::new();
                new_tree.push(ExprKind::Const(1.0));
                Some(new_tree)
            }
            _ => None,
        }
    }

    fn treeify_node(&self, id: NodeId) -> Tree {
        match self.tree.node(id).kind {
            ExprKind::ROOT => {
                let mut new_tree = Tree::new();
                for &child in self.tree.node(id).children() {
                    new_tree.push_tree(self.treeify_node(child));
                }
                new_tree
            }
            x @ (ExprKind::Var(_) | ExprKind::Const(_)) => {
                let mut new_tree = Tree::new();
                new_tree.push(x);
                new_tree
            }
            kind @ (ExprKind::Add | ExprKind::Mul) => {
                let mut new_tree = Tree::new();
                new_tree.start_node(kind);
                for tre in self
                    .tree
                    .node(id)
                    .children()
                    .iter()
                    .map(|&id| self.treeify_node(id))
                {
                    new_tree.push_tree(tre);
                }
                new_tree.finish_node();
                new_tree
            }
        }
    }

    pub fn simplify(self) -> ExprKind {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExprKind {
    ROOT,
    Var(Var),
    Const(f64),
    Add,
    Mul,
}

pub trait Expressable: Into<Expression> {}
impl<T> Expressable for T where T: Into<Expression> {}

impl From<f64> for Expression {
    fn from(value: f64) -> Self {
        let mut tree = Tree::new();
        tree.push(ExprKind::Const(value));
        Expression { tree }
    }
}

impl<T> Add<T> for Expression
where
    T: Expressable,
{
    type Output = Expression;
    fn add(self, rhs: T) -> Self::Output {
        let mut tree = Tree::new();
        tree.start_node(ExprKind::Add);
        tree.push_tree(self.tree);
        tree.push_tree(rhs.into().tree);
        tree.finish_node();
        Expression { tree }
    }
}

impl<T> Mul<T> for Expression
where
    T: Expressable,
{
    type Output = Expression;
    fn mul(self, rhs: T) -> Self::Output {
        let mut tree = Tree::new();
        tree.start_node(ExprKind::Mul);
        tree.push_tree(self.tree);
        tree.push_tree(rhs.into().tree);
        tree.finish_node();
        Expression { tree }
    }
}
