pub mod var;
use crate::tree::*;
pub use var::*;

use std::ops::{Add, Div, Mul, Neg, Sub};

#[macro_export]
macro_rules! e {
    ($ex:expr) => {
        $crate::expression::Expression::from($ex)
    };
}

pub use e;

#[derive(Debug, Clone)]
pub struct Expressand {
    pub(crate) tree: Tree,
}

impl Expressand {
    fn diff_rec(&self, id: NodeId, x: Var) -> Tree {
        match self.tree.node(id).kind {
            ExprKind::ROOT => {
                let mut trees = self
                    .tree
                    .node(id)
                    .children()
                    .iter()
                    .map(|&id| self.diff_rec(id, x))
                    .collect::<Vec<_>>();

                match trees[..] {
                    [_] | [] => trees.pop().unwrap_or_else(|| {
                        let mut new_tree = Tree::new();
                        new_tree.push(ExprKind::Const(0.0));
                        new_tree
                    }),
                    _ => {
                        let mut tree = Tree::new();
                        for tre in trees {
                            tree.push_tree(tre);
                        }
                        tree
                    }
                }
            }
            ExprKind::Add => {
                let mut trees = self
                    .tree
                    .node(id)
                    .children()
                    .iter()
                    .map(|&id| self.diff_rec(id, x))
                    .collect::<Vec<_>>();

                match trees[..] {
                    [_] | [] => trees.pop().unwrap_or_else(|| {
                        let mut new_tree = Tree::new();
                        new_tree.push(ExprKind::Const(0.0));
                        new_tree
                    }),
                    _ => {
                        let mut tree = Tree::new();
                        tree.start_node(ExprKind::Add);
                        for tre in trees {
                            tree.push_tree(tre);
                        }
                        tree.finish_node();
                        tree
                    }
                }
            }
            ExprKind::Mul => {
                let children = self.tree.node(id).children();
                let mut trees = children
                    .iter()
                    .enumerate()
                    .map(|(i, &id)| (i, self.diff_rec(id, x)))
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
                    [_] | [] => trees.pop().unwrap_or_else(|| {
                        let mut new_tree = Tree::new();
                        new_tree.push(ExprKind::Const(0.0));
                        new_tree
                    }),
                    _ => {
                        let mut tree = Tree::new();
                        tree.start_node(ExprKind::Add);
                        for tre in trees {
                            tree.push_tree(tre);
                        }
                        tree.finish_node();
                        tree
                    }
                }
            }
            ExprKind::Ln => {
                let mut iter = self.tree.node(id).children().iter();
                let arg = *iter.next().unwrap();
                assert!(iter.next().is_none());

                let arg_tree = treeify_node(&self.tree, arg);

                let d_arg = self.diff_rec(arg, x);

                let mut new_tree = Tree::new();
                new_tree.start_node(ExprKind::Mul);
                new_tree.push_tree(d_arg);

                new_tree.start_node(ExprKind::Exp);

                new_tree.start_node(ExprKind::Mul);
                new_tree.start_node(ExprKind::Ln);
                new_tree.push_tree(arg_tree);
                new_tree.finish_node();
                new_tree.push(ExprKind::Const(-1.0));
                new_tree.finish_node();

                new_tree.finish_node();

                new_tree.finish_node();
                new_tree
            }
            ExprKind::Var(v) if v.id == x.id => {
                let mut new_tree = Tree::new();
                new_tree.push(ExprKind::Const(1.0));
                new_tree
            }
            ExprKind::Var(_) | ExprKind::Const(_) => {
                let mut new_tree = Tree::new();
                new_tree.push(ExprKind::Const(0.0));
                new_tree
            }
            ExprKind::Exp => {
                let mut iter = self.tree.node(id).children().iter();
                let exp = *iter.next().unwrap();
                assert!(iter.next().is_none());

                let node = treeify_node(&self.tree, id);
                let d_exp = self.diff_rec(exp, x);

                let mut new_tree = Tree::new();
                new_tree.start_node(ExprKind::Mul);
                new_tree.push_tree(node);
                new_tree.push_tree(d_exp);
                new_tree.finish_node();
                new_tree
            }
        }
    }

    fn simplify_rec(&self, id: NodeId) -> Tree {
        match self.tree.node(id).kind {
            ExprKind::ROOT => {
                let mut new_tree = Tree::new();
                for &child in self.tree.node(id).children() {
                    new_tree.push_tree(self.simplify_rec(child));
                }
                new_tree
            }
            ExprKind::Ln => {
                let mut iter = self.tree.node(id).children().iter();
                let mut tree = Tree::new();
                let arg = *iter.next().unwrap();
                assert!(iter.next().is_none());
                tree.start_node(ExprKind::Ln);
                tree.push_tree(self.simplify_rec(arg));
                tree.finish_node();
                tree
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
                    .map(|&id| self.simplify_rec(id))
                {
                    match tree.sub_roots()[..] {
                        [Node {
                            kind: ExprKind::Add,
                            children,
                            ..
                        }] => {
                            for &child in children {
                                trees.push(treeify_node(&tree, child));
                            }
                        }
                        _ => trees.push(tree),
                    }
                }
                let mut consts = 0.0;
                trees.retain(|tree| {
                    if let [Node {
                        kind: ExprKind::Const(x),
                        ..
                    }] = tree.sub_roots()[..]
                    {
                        consts += x;
                        false
                    } else {
                        true
                    }
                });
                if consts != 0.0 {
                    trees.push({
                        let mut new_tree = Tree::new();
                        new_tree.push(ExprKind::Const(consts));
                        new_tree
                    });
                }
                match trees.len() {
                    0 | 1 => trees.pop().unwrap_or_else(|| {
                        let mut new_tree = Tree::new();
                        new_tree.push(ExprKind::Const(0.0));
                        new_tree
                    }),
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
                    .map(|&id| self.simplify_rec(id))
                {
                    match tree.sub_roots()[..] {
                        [Node {
                            kind: ExprKind::Mul,
                            children,
                            ..
                        }] => {
                            for &child in children {
                                trees.push(treeify_node(&tree, child));
                            }
                        }
                        [Node {
                            kind: ExprKind::Const(x),
                            ..
                        }] if *x == 0.0 => {
                            trees.clear();
                            let mut new_tree = Tree::new();
                            new_tree.push(ExprKind::Const(0.0));
                            trees.push(new_tree);
                            break;
                        }
                        _ => trees.push(tree),
                    }
                }
                let mut consts = 1.0;
                trees.retain(|tree| {
                    if let [Node {
                        kind: ExprKind::Const(x),
                        ..
                    }] = tree.sub_roots()[..]
                    {
                        consts *= x;
                        false
                    } else {
                        true
                    }
                });
                if consts != 1.0 {
                    trees.push({
                        let mut new_tree = Tree::new();
                        new_tree.push(ExprKind::Const(consts));
                        new_tree
                    });
                }

                match trees.len() {
                    0 | 1 => {
                        return trees.pop().unwrap_or_else(|| {
                            let mut new_tree = Tree::new();
                            new_tree.push(ExprKind::Const(1.0));
                            new_tree
                        })
                    }
                    _ => (),
                }

                let mut adds = Vec::new();
                let mut others = Vec::new();
                for tree in trees {
                    if tree.sub_roots()[0].kind == ExprKind::Add {
                        let terms = tree.sub_roots()[0]
                            .children()
                            .iter()
                            .map(|&id| treeify_node(&tree, id))
                            .collect::<Vec<_>>();
                        adds.push(terms);
                    } else {
                        others.push(tree);
                    }
                }

                fn recurse_open<'a>(
                    arr: &'a [Vec<Tree>],
                    idx: usize,
                    curr: &mut Vec<&'a Tree>,
                    trees: &mut Vec<Vec<Tree>>,
                ) {
                    if idx >= arr.len() {
                        return trees.push(curr.iter().cloned().cloned().collect());
                    }
                    for i in &arr[idx] {
                        curr.push(i);
                        recurse_open(arr, idx + 1, curr, trees);
                        curr.pop();
                    }
                }

                let mut terms = Vec::new();
                recurse_open(&adds, 0, &mut Vec::new(), &mut terms);

                let mut new_tree = Tree::new();
                new_tree.start_node(ExprKind::Add);
                for trees in terms {
                    new_tree.start_node(ExprKind::Mul);
                    for tree in trees {
                        new_tree.push_tree(tree);
                    }
                    for tree in &others {
                        new_tree.push_tree(tree.clone());
                    }
                    new_tree.finish_node();
                }
                new_tree.finish_node();
                new_tree
            }
            ExprKind::Exp => {
                let mut iter = self.tree.node(id).children().iter();
                let exp = *iter.next().unwrap();
                assert!(iter.next().is_none());

                let exp_tree = self.simplify_rec(exp);

                if let [node] = exp_tree.sub_roots()[..] {
                    match node.kind {
                        ExprKind::Const(c) if c == 0.0 => {
                            let mut new_tree = Tree::new();
                            new_tree.push(ExprKind::Const(1.0));
                            return new_tree;
                        }
                        ExprKind::Ln => {
                            let mut iter = self.tree.node(id).children().iter();
                            let mut tree = Tree::new();
                            let arg = *iter.next().unwrap();
                            assert!(iter.next().is_none());
                            tree.push_tree(self.simplify_rec(arg));
                            return tree;
                        }
                        _ => (),
                    }
                }

                let mut new_tree = Tree::new();
                new_tree.start_node(ExprKind::Exp);
                new_tree.push_tree(exp_tree);
                new_tree.finish_node();
                new_tree
            }
        }
    }

    fn eval_rec(&self, id: NodeId, x: &[f64]) -> f64 {
        match self.tree.node(id).kind {
            ExprKind::ROOT => self
                .tree
                .node(id)
                .children()
                .iter()
                .map(|&id| self.eval_rec(id, x))
                .sum(),
            ExprKind::Ln => {
                let mut iter = self.tree.node(id).children().iter();

                let arg = *iter.next().unwrap();
                assert!(iter.next().is_none());

                let arg = self.eval_rec(arg, x);
                arg.ln()
            }
            ExprKind::Var(v) => x[v.id],
            ExprKind::Const(c) => c,
            ExprKind::Add => self
                .tree
                .node(id)
                .children()
                .iter()
                .map(|&id| self.eval_rec(id, x))
                .sum(),
            ExprKind::Mul => self
                .tree
                .node(id)
                .children()
                .iter()
                .map(|&id| self.eval_rec(id, x))
                .product(),
            ExprKind::Exp => {
                let mut iter = self.tree.node(id).children().iter();
                let exp = *iter.next().unwrap();
                assert!(iter.next().is_none());
                let exp = self.eval_rec(exp, x);
                exp.exp()
            }
        }
    }

    pub(crate) fn eval(&self, x: &[f64]) -> f64 {
        self.eval_rec(NodeId::ROOT, x)
    }
}

#[track_caller]
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
        kind @ (ExprKind::Add | ExprKind::Mul | ExprKind::Exp | ExprKind::Ln) => {
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExprKind {
    ROOT,
    Var(Var),
    Const(f64),
    Add,
    Mul,
    Exp,
    Ln,
}

#[derive(Debug, Clone)]
pub struct Expressable<T: Clone>(T);

impl<T: Clone> Expressable<T>
where
    Expression: From<Expressable<T>>,
{
    pub fn diff(self, x: Var) -> Expression {
        Expressable(Expressand {
            tree: e!(self).0.diff_rec(NodeId::ROOT, x),
        })
    }

    pub fn simplify(self) -> Expression {
        Expressable(Expressand {
            tree: e!(self).0.simplify_rec(NodeId::ROOT),
        })
    }

    pub fn ln(self) -> Expression {
        let mut tree = Tree::new();
        tree.start_node(ExprKind::Ln);
        tree.push_tree(e!(self).0.tree);
        tree.finish_node();
        Expressable(Expressand { tree })
    }

    pub fn exp(self) -> Expression {
        let mut tree = Tree::new();
        tree.start_node(ExprKind::Exp);
        tree.push_tree(e!(self).0.tree);
        tree.finish_node();
        Expressable(Expressand { tree })
    }

    pub fn pow<U>(self, exp: U) -> Expression
    where
        Expression: From<U>,
    {
        let mut tree = Tree::new();
        tree.start_node(ExprKind::Exp);

        tree.start_node(ExprKind::Mul);
        tree.start_node(ExprKind::Ln);
        tree.push_tree(e!(self).0.tree);
        tree.finish_node();
        tree.push_tree(e!(exp).0.tree);
        tree.finish_node();

        tree.finish_node();
        Expressable(Expressand { tree })
    }

    pub fn log<U>(self, base: U) -> Expression
    where
        Expression: From<U>,
    {
        let mut tree = Tree::new();
        tree.start_node(ExprKind::Mul);

        tree.start_node(ExprKind::Ln);
        tree.push_tree(e!(self).0.tree);
        tree.finish_node();

        tree.start_node(ExprKind::Exp);
        tree.start_node(ExprKind::Mul);
        tree.start_node(ExprKind::Ln);
        tree.start_node(ExprKind::Ln);
        tree.push_tree(e!(base).0.tree);
        tree.finish_node();
        tree.finish_node();
        tree.push(ExprKind::Const(-1.0));
        tree.finish_node();
        tree.finish_node();

        tree.finish_node();
        Expressable(Expressand { tree })
    }

    pub(crate) fn tree(self) -> Tree {
        e!(self.clone()).0.tree
    }

    pub(crate) fn ex(self) -> Expressand {
        e!(self).0
    }
}

pub type Expression = Expressable<Expressand>;

impl From<Expressand> for Expression {
    fn from(value: Expressand) -> Self {
        Expressable(value)
    }
}

impl<T: Into<f64>> From<T> for Expression {
    fn from(value: T) -> Self {
        let mut tree = Tree::new();
        tree.push(ExprKind::Const(value.into()));
        Expressable(Expressand { tree })
    }
}

impl<T: Clone, U: Clone> Add<Expressable<U>> for Expressable<T>
where
    Expression: From<Expressable<T>>,
    Expression: From<Expressable<U>>,
{
    type Output = Expression;
    fn add(self, rhs: Expressable<U>) -> Self::Output {
        let mut tree = Tree::new();
        tree.start_node(ExprKind::Add);
        tree.push_tree(e!(self).0.tree);
        tree.push_tree(e!(rhs).0.tree);
        tree.finish_node();
        Expressable(Expressand { tree })
    }
}

impl<T: Clone, U: Clone> Mul<Expressable<U>> for Expressable<T>
where
    Expression: From<Expressable<T>>,
    Expression: From<Expressable<U>>,
{
    type Output = Expression;
    fn mul(self, rhs: Expressable<U>) -> Self::Output {
        let mut tree = Tree::new();
        tree.start_node(ExprKind::Mul);
        tree.push_tree(e!(self).0.tree);
        tree.push_tree(e!(rhs).0.tree);
        tree.finish_node();
        Expressable(Expressand { tree })
    }
}

impl<T: Clone, U: Clone> Sub<Expressable<U>> for Expressable<T>
where
    Expression: From<Expressable<T>>,
    Expression: From<Expressable<U>>,
{
    type Output = Expression;
    fn sub(self, rhs: Expressable<U>) -> Self::Output {
        let mut tree = Tree::new();
        tree.start_node(ExprKind::Add);
        tree.push_tree(e!(self).0.tree);
        tree.start_node(ExprKind::Mul);
        tree.push(ExprKind::Const(-1.0));
        tree.push_tree(e!(rhs).0.tree);
        tree.finish_node();
        tree.finish_node();
        Expressable(Expressand { tree })
    }
}

impl<T: Clone, U: Clone> Div<Expressable<U>> for Expressable<T>
where
    Expression: From<Expressable<T>>,
    Expression: From<Expressable<U>>,
{
    type Output = Expression;
    fn div(self, rhs: Expressable<U>) -> Self::Output {
        let mut tree = Tree::new();
        tree.start_node(ExprKind::Mul);
        tree.push_tree(e!(self).0.tree);
        tree.start_node(ExprKind::Exp);

        tree.start_node(ExprKind::Mul);
        tree.start_node(ExprKind::Ln);
        tree.push_tree(e!(rhs).0.tree);
        tree.finish_node();
        tree.push(ExprKind::Const(-1.0));
        tree.finish_node();

        tree.finish_node();
        tree.finish_node();
        Expressable(Expressand { tree })
    }
}

impl<T: Clone> Neg for Expressable<T>
where
    Expression: From<Expressable<T>>,
{
    type Output = Expression;
    fn neg(self) -> Self::Output {
        let mut tree = Tree::new();
        tree.start_node(ExprKind::Mul);
        tree.push(ExprKind::Const(-1.0));
        tree.push_tree(e!(self).0.tree);
        tree.finish_node();
        Expressable(Expressand { tree })
    }
}

impl PartialEq for Expressand {
    fn eq(&self, _other: &Self) -> bool {
        false
    }
}
