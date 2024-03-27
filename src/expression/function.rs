use num_complex::Complex64;

use crate::{ExprKind, Tree, NEGONE};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Function {
    Sin,
    Cos,
}

impl Function {
    pub fn eval(&self, args: &[Complex64]) -> Complex64 {
        match self {
            Function::Sin => args[0].sin(),
            Function::Cos => args[0].cos(),
        }
    }

    pub fn diff(&self, args: Vec<Tree>) -> Tree {
        match self {
            Function::Sin => {
                let mut tree = Tree::new();
                tree.start_node(ExprKind::Func(Function::Cos));
                for arg in args {
                    tree.push_tree(arg);
                }
                tree.finish_node();
                tree
            }
            Function::Cos => {
                let mut tree = Tree::new();
                tree.start_node(ExprKind::Mul);
                tree.push(ExprKind::Const(NEGONE));
                tree.start_node(ExprKind::Func(Function::Sin));
                for arg in args {
                    tree.push_tree(arg);
                }
                tree.finish_node();
                tree.finish_node();
                tree
            }
        }
    }

    pub fn str(&self) -> &'static str {
        match self {
            Function::Sin => "sin",
            Function::Cos => "cos",
        }
    }
}
