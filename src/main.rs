#![allow(dead_code, non_snake_case)]
use std::f64::consts::PI;

use symrs::*;
mod curvature;

fn main() {
    let mut sys = System::default();
    let [x] = sys.symbols("x").unwrap();

    let y = asin(x);
    let dy = y.clone().diff(x);

    println!("{}", sys.str(y.clone()));
    println!("{}", sys.str(dy.clone()));

    let v = [c!(PI / 8.0)];
    printc(sys.eval(y, v));
    printc(sys.eval(dy, v));
}
