#![allow(dead_code, non_snake_case)]
use std::f64::consts::PI;

use symrs::*;
mod tensor;
mod curvature;

fn main() {
    let mut sys = System::default();
    let [ct, s] = sys.symbols("ct s").unwrap();
    let z = (e!(ct).pow(2.0) -  e!(s).pow(2.0)).pow(0.5);

    let A = e!(ct) * ((e!(ct) + z.clone()) /  e!(s)).ln() - z;
    let E = A.clone().diff(ct).simplify();

    println!("{}", sys.str(A.clone()));
    println!("{}", sys.str(E.clone()));

    // let v = [c!(PI / 4.0)];
    // printc(sys.eval(y, v));
    // printc(sys.eval(dy, v));
}
