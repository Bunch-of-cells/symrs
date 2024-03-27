pub mod expression;
pub mod matrix;
pub mod tree;

use crate::tree::{NodeId, Tree};
pub use expression::*;
pub use matrix::*;

#[derive(Default, Clone, Debug)]
pub struct System {
    variables: Vec<String>,
}

impl System {
    pub fn symbols<const N: usize>(&mut self, idents: &str) -> Result<[Var; N], String> {
        let idents = idents.split_ascii_whitespace().collect::<Vec<_>>();
        assert!(idents.len() == N, "Inadequate amount of identifiers");
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

    pub fn str<T: Clone>(&self, exp: Expressable<T>) -> String
    where
        Expression: From<Expressable<T>>,
    {
        let tree = exp.tree();
        let mut f = String::new();
        fn write_children(vars: &[String], tree: &Tree, id: NodeId, f: &mut String) {
            match tree.node(id).kind() {
                ExprKind::Var(x) => *f += &vars[x.id],
                ExprKind::Const(c) => f.push_str(&format!("{c:.3}")),
                ExprKind::Add => {
                    *f += "(";
                    let mut iter = tree.node(id).children().iter();
                    write_children(vars, tree, *iter.next().unwrap(), f);
                    for &child in iter {
                        *f += "+";
                        write_children(vars, tree, child, f);
                    }
                    *f += ")";
                }
                ExprKind::Mul => {
                    *f += " ";
                    let mut iter = tree.node(id).children().iter();
                    write_children(vars, tree, *iter.next().unwrap(), f);
                    for &child in iter {
                        *f += "*";
                        write_children(vars, tree, child, f);
                    }
                    *f += " ";
                }
                ExprKind::Exp => {
                    *f += " e^";
                    let mut iter = tree.node(id).children().iter();
                    write_children(vars, tree, *iter.next().unwrap(), f);
                    assert!(iter.next().is_none());
                    *f += " ";
                }
                ExprKind::Ln => {
                    *f += "ln(";
                    let mut iter = tree.node(id).children().iter();
                    write_children(vars, tree, *iter.next().unwrap(), f);
                    assert!(iter.next().is_none());
                    *f += ")";
                }
                ExprKind::ROOT => {
                    for &child in tree.node(id).children() {
                        write_children(vars, tree, child, f);
                    }
                }
                ExprKind::Func(func) => {
                    *f += func.str();
                    *f += "(";
                    let mut iter = tree.node(id).children().iter();
                    write_children(vars, tree, *iter.next().unwrap(), f);
                    for &child in iter {
                        *f += ",";
                        write_children(vars, tree, child, f);
                    }
                    *f += ")";
                }
            }
        }
        write_children(&self.variables, &tree, NodeId::ROOT, &mut f);
        f
    }

    pub fn strmat<const N: usize>(&self, mat: SqMatrix<N>) -> String {
        let mut f = String::new();
        for x in mat.0.into_iter() {
            f += "[";
            for x in x.into_iter() {
                f.push_str(&format!("{:^8}", self.str(x)));
            }
            f += "]\n";
        }
        f
    }

    pub fn eval<const N: usize, T: Clone>(&self, exp: Expressable<T>, x: [f64; N]) -> f64
    where
        Expression: From<Expressable<T>>,
    {
        assert!(self.variables.len() == N, "Inadequate number of values provided");
        exp.ex().eval(&x)
    }

    pub fn evalmat<const N: usize, const M: usize>(
        &self,
        mut mat: SqMatrix<M>,
        x: [f64; N],
    ) -> SqMatrix<M> {
        assert!(self.variables.len() == N);
        for i in mat.0.iter_mut() {
            for j in i.iter_mut() {
                *j = j.clone().ex().eval(&x).into();
            }
        }
        mat
    }
}
