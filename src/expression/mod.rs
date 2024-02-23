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

    pub fn simplify(&self) -> Expression {
        Expression {
            tree: self.simplify_inner(NodeId::ROOT),
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
                            new_tree.push_tree(treeify_node(&self.tree, *child));
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
            ExprKind::Var(_) | ExprKind::Const(_) => {
                let mut new_tree = Tree::new();
                new_tree.push(ExprKind::Const(0.0));
                Some(new_tree)
            } // _ => None,
        }
    }

    pub fn simplify_inner(&self, id: NodeId) -> Tree {
        match self.tree.node(id).kind {
            ExprKind::ROOT => {
                let mut new_tree = Tree::new();
                for &child in self.tree.node(id).children() {
                    new_tree.push_tree(self.simplify_inner(child));
                }
                new_tree
            }
            x @ (ExprKind::Var(_) | ExprKind::Const(_)) => {
                let mut new_tree = Tree::new();
                new_tree.push(x);
                new_tree
            }
            ExprKind::Add => {
                let mut trees = Vec::new();
                for tree in self
                    .tree
                    .node(id)
                    .children()
                    .iter()
                    .map(|&id| self.simplify_inner(id))
                    .filter(|tree| match tree.root().children()[..] {
                        [id] => tree.node(id).kind != ExprKind::Const(0.0),
                        _ => true,
                    })
                {
                    trees.push(tree);
                }
                match trees.len() {
                    0 => {
                        let mut new_tree = Tree::new();
                        new_tree.push(ExprKind::Const(0.0));
                        new_tree
                    }
                    1 => trees.pop().unwrap(),
                    _ => {
                        let mut new_tree = Tree::new();
                        new_tree.start_node(ExprKind::Add);
                        for tree in trees {
                            new_tree.push_tree(tree);
                        }
                        new_tree.finish_node();
                        new_tree
                    }
                }
            }
            ExprKind::Mul => {
                let mut trees = Vec::new();
                for tree in self
                    .tree
                    .node(id)
                    .children()
                    .iter()
                    .map(|&id| self.simplify_inner(id))
                    .filter(|tree| match tree.root().children()[..] {
                        [id] => tree.node(id).kind != ExprKind::Const(1.0),
                        _ => true,
                    })
                {
                    match tree.root().children()[..] {
                        [id] if tree.node(id).kind == ExprKind::Const(0.0) => {
                            trees.clear();
                            trees.push({
                                let mut tree = Tree::new();
                                tree.push(ExprKind::Const(0.0));
                                tree
                            });
                            break;
                        }
                        _ => trees.push(tree),
                    }
                }
                match trees.len() {
                    0 => {
                        let mut new_tree = Tree::new();
                        new_tree.push(ExprKind::Const(0.0));
                        new_tree
                    }
                    1 => trees.pop().unwrap_or_default(),
                    _ => {
                        let mut new_tree = Tree::new();
                        new_tree.start_node(ExprKind::Mul);
                        for tree in trees {
                            new_tree.push_tree(tree);
                        }
                        new_tree.finish_node();
                        new_tree
                    }
                }
            }
        }
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

fn treeify_node(tree: &Tree, id: NodeId) -> Tree {
    match tree.node(id).kind {
        ExprKind::ROOT => {
            let mut new_tree = Tree::new();
            for &child in tree.node(id).children() {
                new_tree.push_tree(treeify_node(tree, child));
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
            for tre in tree
                .node(id)
                .children()
                .iter()
                .map(|&id| treeify_node(tree, id))
            {
                new_tree.push_tree(tre);
            }
            new_tree.finish_node();
            new_tree
        }
    }
}
