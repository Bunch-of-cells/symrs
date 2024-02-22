pub mod expression;

pub use expression::*;

use crate::tree::{NodeId, Tree, TreeElement};
pub mod tree;

#[derive(Default, Clone, Debug)]
pub struct System {
    variables: Vec<String>,
}

impl System {
    pub fn symbols<const N: usize>(&mut self, idents: &str) -> Result<[Var; N], String> {
        let idents = idents.split_ascii_whitespace().collect::<Vec<_>>();
        assert!(idents.len() == N);
        let mut vars: [Var; N] = std::array::from_fn(|_| Var { id: 0 });

        for (i, &ident) in idents.iter().enumerate() {
            if self.variables.iter().any(|var| var == ident) {
                return Err(String::from("Alr exists"));
            }
            let var = Var {
                id: self.variables.len(),
            };
            self.variables.push(ident.to_string());
            vars[i] = var;
        }
        Ok(vars)
    }

    pub fn str<T: Expressable>(&self, exp: T) -> String {
        let exp: Expression = exp.into();
        let mut f = String::new();
        fn write_children(vars: &[String], tree: &Tree, id: NodeId, f: &mut String) {
            for child in tree.node(id).children_with_leaves(tree) {
                match child {
                    TreeElement::Leaf(l) => {
                        match tree.leaf(l).kind {
                            ExprKind::Var(x) => *f += &vars[x.id],
                            ExprKind::Const(c) => f.push_str(&c.to_string()),
                            _ => *f += ":",
                        };
                    }
                    TreeElement::Node(n) => {
                        match tree.node(n).kind {
                            ExprKind::Add => *f += "+",
                            ExprKind::Mul => *f += "*",
                            _ => *f += ":",
                        };
                        write_children(vars, tree, n, f)
                    }
                }
            }
        }
        write_children(&self.variables, &exp.tree, NodeId::ROOT, &mut f);
        f
    }
}
