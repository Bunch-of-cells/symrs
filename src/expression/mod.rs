pub mod var;
use crate::tree::*;
pub use var::*;

use std::ops::{Add, Mul};

#[derive(Debug, Clone)]
pub struct Expression {
    pub(crate) tree: Tree,
}

impl Expression {
    pub fn d(self, x: Var) -> Expression {
        fn d_inner(tree: &Tree, id: NodeId, x: Var) -> Option<Tree> {
            match tree.node(id).kind {
                ExprKind::Root => {
                    let mut new_tree = Tree::new();
                    for child in tree.node(id).children_with_leaves(tree) {
                        if let Some(tre) = match child {
                            TreeElement::Leaf(l) => match tree.leaf(l).kind {
                                ExprKind::Var(ref v) => {
                                    if v.id == x.id {
                                        Some({
                                            let mut tree = Tree::new();
                                            tree.push(ExprKind::Const(1.0));
                                            tree
                                        })
                                    } else {
                                        None
                                    }
                                }
                                ExprKind::Const(_) => None,
                                _ => unreachable!(),
                            },
                            TreeElement::Node(n) => d_inner(tree, n, x.clone()),
                        } {
                            new_tree.push_tree(tre);
                        }
                    }
                    if new_tree
                        .node(NodeId::ROOT)
                        .children_with_leaves(&new_tree)
                        .next()
                        .is_none()
                    {
                        None
                    } else {
                        Some(new_tree)
                    }
                }
                ExprKind::Add => {
                    let mut trees = tree
                        .node(id)
                        .children()
                        .iter()
                        .filter_map(|&id| d_inner(tree, id, x.clone()))
                        .collect::<Vec<_>>();

                    match trees[..] {
                        [_] | [] => trees.pop(),
                        _ => {
                            let mut tree = Tree::new();
                            tree.start_node(ExprKind::Add);
                            for tre in trees {
                                tree.push_tree(tre);
                            }
                            Some(tree)
                        }
                    }
                }
                ExprKind::Mul => {
                    let children = tree.node(id).children();
                    let mut trees = children
                        .iter()
                        .enumerate()
                        .filter_map(|(i, &id)| d_inner(tree, id, x.clone()).map(|d| (i, d)))
                        .map(|(i, d)| {
                            let mut new_tree = Tree::new();
                            new_tree.start_node(ExprKind::Mul);
                            new_tree.push_tree(d);
                            for (j, child) in children.iter().enumerate() {
                                if i == j {
                                    continue;
                                }
                                new_tree.push_tree(recreate_node(tree, *child));
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
                            Some(tree)
                        }
                    }
                }
                _ => unreachable!(),
            }
        }

        fn recreate_node(tree: &Tree, id: NodeId) -> Tree {
            match tree.node(id).kind {
                ExprKind::Root => {
                    let mut new_tree = Tree::new();
                    for child in tree.node(id).children_with_leaves(tree) {
                        new_tree.push_tree(match child {
                            TreeElement::Leaf(l) => {
                                let mut new_tree = Tree::new();
                                new_tree.push(tree.leaf(l).kind);
                                new_tree
                            }
                            TreeElement::Node(n) => recreate_node(tree, n),
                        });
                    }
                    new_tree
                }
                kind @ (ExprKind::Add | ExprKind::Mul) => {
                    let mut new_tree = Tree::new();
                    new_tree.start_node(kind);
                    for tre in tree
                        .node(id)
                        .children()
                        .iter()
                        .map(|&id| recreate_node(tree, id))
                    {
                        new_tree.push_tree(tre);
                    }
                    new_tree
                }
                _ => unreachable!(),
            }
        }
        Expression {
            tree: d_inner(&self.tree, NodeId::ROOT, x).unwrap_or_default(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ExprKind {
    Var(Var),
    Const(f64),
    Root,
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
