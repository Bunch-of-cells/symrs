#![allow(dead_code, non_snake_case)]

use symrs::System;
mod curvature;

fn main() {
    let mut sys = System::default();
    let [x, y, z, w] = sys.symbols("x y z w").unwrap();
    let a = (x + y + z) * (x + y - w);
    println!("{}", sys.str(&a));
    println!("{}", sys.str(&a.simplify()));
}
