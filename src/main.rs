#![allow(dead_code, non_snake_case)]

use std::f32::consts::E;

use symrs::{Expressable, SqMatrix, System};
mod curvature;

fn main() {
    let mut sys = System::default();
    let [r] = sys.symbols("r").unwrap();
    let a = (-4).ex() / r * E.ex().pow(-r);
    let g = SqMatrix([
        [a.clone(), 0.ex(), 0.ex(), 0.ex()],
        [0.ex(), -a, 0.ex(), 0.ex()],
        [0.ex(), 0.ex(), r.ex().pow(2.ex()), 0.ex()],
        [0.ex(), 0.ex(), 0.ex(), r.ex().pow(2.ex())],
    ]);
    println!("{}", sys.strmat(&g.clone().simplify()));
    println!("{}", sys.strmat(&g.inv().simplify()));

    let x = [0.0001];

    println!("{}", sys.strmat(&sys.evalmat(g.clone(), x)));
    println!("{}", sys.strmat(&sys.evalmat(g.inv(), x)));
}
