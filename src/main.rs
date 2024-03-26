#![allow(dead_code, non_snake_case)]

use symrs::*;
mod curvature;

fn main() {
    let mut sys = System::default();
    let [x] = sys.symbols("x").unwrap();

    let y = e!(x).log(2.0).simplify();
    let dy = y.clone().diff(x).simplify();

    println!("{}", sys.str(y.clone()));
    println!("{}", sys.str(dy.clone()));

    let x = [2.0];
    println!("{}", sys.eval(y, x));
    println!("{}", sys.eval(dy, x));
}
