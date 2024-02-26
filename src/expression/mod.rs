pub mod var;
use crate::tree::*;
pub use var::*;

use std::ops::{Add, Div, Mul, Neg, Sub};

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
            }
            ExprKind::Pow => {
                let mut iter = self.tree.node(id).children().iter();
                let base = *iter.next().unwrap();
                let exp = *iter.next().unwrap();
                assert!(iter.next().is_none());

                let base_tree = treeify_node(&self.tree, base);
                let exp_tree = treeify_node(&self.tree, exp);

                let d_exp = self.d_inner(exp, x);
                match d_exp {
                    None => (),
                    Some(d_exp) => {
                        let d_exp = Expression { tree: d_exp }.simplify().tree;
                        if d_exp.node(*d_exp.root().children().first().unwrap()).kind
                            != ExprKind::Const(0.0)
                        {
                            panic!("NOT IMPLEMENTING LOG")
                        }
                    }
                }
                let d_base = self.d_inner(base, x)?;

                let mut new_tree = Tree::new();
                new_tree.start_node(ExprKind::Mul);

                new_tree.start_node(ExprKind::Pow);
                new_tree.push_tree(base_tree);
                new_tree.start_node(ExprKind::Add);
                new_tree.push_tree(exp_tree.clone());
                new_tree.push(ExprKind::Const(-1.0));
                new_tree.finish_node();
                new_tree.finish_node();

                new_tree.push_tree(exp_tree);
                new_tree.push_tree(d_base);

                new_tree.finish_node();
                Some(new_tree)
            }
        }
    }

    fn simplify_inner(&self, id: NodeId) -> Tree {
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
                    .map(|&id| self.simplify_inner(id))
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
            ExprKind::Pow => {
                let mut iter = self.tree.node(id).children().iter();

                let base = *iter.next().unwrap();
                let exp = *iter.next().unwrap();
                assert!(iter.next().is_none());

                let mut base_tree = self.simplify_inner(base);
                let mut exp_tree = self.simplify_inner(exp);

                let mut b = None;
                if let [id] = base_tree.root().children()[..] {
                    match base_tree.node(id).kind {
                        ExprKind::Const(c) if c == 0.0 || c == 1.0 => {
                            return base_tree;
                        }
                        ExprKind::Const(c) => {
                            b = Some(c);
                        }
                        ExprKind::Pow => {
                            let mut iter = base_tree.node(id).children().iter();
                            let base = *iter.next().unwrap();
                            let exp = *iter.next().unwrap();
                            let new_exp = treeify_node(&base_tree, exp);
                            base_tree = treeify_node(&base_tree, base);
                            let mut new_exp_tree = Tree::new();
                            new_exp_tree.start_node(ExprKind::Mul);
                            new_exp_tree.push_tree(exp_tree);
                            new_exp_tree.push_tree(new_exp);
                            new_exp_tree.finish_node();
                            exp_tree = Expression { tree: new_exp_tree }.simplify().tree;
                        }
                        _ => (),
                    }
                }

                if let [node] = exp_tree.sub_roots()[..] {
                    match node.kind {
                        ExprKind::Const(c) if c == 0.0 => {
                            let mut new_tree = Tree::new();
                            new_tree.push(ExprKind::Const(1.0));
                            return new_tree;
                        }
                        ExprKind::Const(c) if c == 1.0 => {
                            return base_tree;
                        }
                        ExprKind::Const(c) if b.is_some() => {
                            let mut new_tree = Tree::new();
                            new_tree.push(ExprKind::Const(b.unwrap().powf(c)));
                            return new_tree;
                        }
                        _ => (),
                    }
                }

                let mut new_tree = Tree::new();
                new_tree.start_node(ExprKind::Pow);
                new_tree.push_tree(base_tree);
                new_tree.push_tree(exp_tree);
                new_tree.finish_node();
                new_tree
            }
        }
    }

    fn eval_inner(&self, id: NodeId, x: &[f64]) -> f64 {
        match self.tree.node(id).kind {
            ExprKind::ROOT => self
                .tree
                .node(id)
                .children()
                .iter()
                .map(|&id| self.eval_inner(id, x))
                .sum(),
            ExprKind::Var(v) => x[v.id],
            ExprKind::Const(c) => c,
            ExprKind::Add => self
                .tree
                .node(id)
                .children()
                .iter()
                .map(|&id| self.eval_inner(id, x))
                .sum(),
            ExprKind::Mul => self
                .tree
                .node(id)
                .children()
                .iter()
                .map(|&id| self.eval_inner(id, x))
                .product(),
            ExprKind::Pow => {
                let mut iter = self.tree.node(id).children().iter();

                let base = *iter.next().unwrap();
                let exp = *iter.next().unwrap();
                assert!(iter.next().is_none());

                let base = self.eval_inner(base, x);
                let exp = self.eval_inner(exp, x);
                base.powf(exp)
            }
        }
    }

    pub(crate) fn eval(&self, x: &[f64]) -> f64 {
        self.eval_inner(NodeId::ROOT, x)
    }

    pub fn pow<T: Expressable>(self, rhs: T) -> Expression {
        let mut tree = Tree::new();
        tree.start_node(ExprKind::Pow);
        tree.push_tree(self.tree);
        tree.push_tree(rhs.into().tree);
        tree.finish_node();
        Expression { tree }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ExprKind {
    ROOT,
    Var(Var),
    Const(f64),
    Add,
    Mul,
    Pow,
}

pub trait Expressable: Into<Expression> + Clone {
    fn ex(self) -> Expression {
        self.into()
    }
}

impl<T> Expressable for T where T: Into<Expression> + Clone {}

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

impl<T> Sub<T> for Expression
where
    T: Expressable,
{
    type Output = Expression;
    fn sub(self, rhs: T) -> Self::Output {
        let mut tree = Tree::new();
        tree.start_node(ExprKind::Add);
        tree.push_tree(self.tree);
        tree.start_node(ExprKind::Mul);
        tree.push(ExprKind::Const(-1.0));
        tree.push_tree(rhs.into().tree);
        tree.finish_node();
        tree.finish_node();
        Expression { tree }
    }
}

impl<T> Div<T> for Expression
where
    T: Expressable,
{
    type Output = Expression;
    fn div(self, rhs: T) -> Self::Output {
        let mut tree = Tree::new();
        tree.start_node(ExprKind::Mul);
        tree.push_tree(self.tree);
        tree.start_node(ExprKind::Pow);
        tree.push_tree(rhs.into().tree);
        tree.push(ExprKind::Const(-1.0));
        tree.finish_node();
        tree.finish_node();
        Expression { tree }
    }
}

impl Neg for Expression {
    type Output = Expression;
    fn neg(self) -> Self::Output {
        let mut tree = Tree::new();
        tree.start_node(ExprKind::Mul);
        tree.push(ExprKind::Const(-1.0));
        tree.push_tree(self.tree);
        tree.finish_node();
        Expression { tree }
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
        kind @ (ExprKind::Add | ExprKind::Mul | ExprKind::Pow) => {
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
