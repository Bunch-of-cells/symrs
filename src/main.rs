#![allow(dead_code, non_snake_case)]

use num_complex::Complex64;
use std::f64::consts::PI;

use symrs::*;
mod curvature;

fn main() {
    let mut sys = System::default();
    let [x] = sys.symbols("x").unwrap();

    let y = sin(x) / cos(x);
    let dy = y.clone().diff(x).simplify();

    println!("{}", sys.str(y.clone()));
    println!("{}", sys.str(dy.clone()));

    let x = [Complex64::new(PI, 0.0)];
    println!("{:.5}", sys.eval(y, x));
    println!("{:.5}", sys.eval(dy, x));
}
