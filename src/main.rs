#![allow(dead_code, non_snake_case)]

use symrs::*;
mod curvature;

fn main() {
    let mut sys = System::default();
    let [r] = sys.symbols("r").unwrap();
    let a = e!(-4) / e!(r) * e!(std::f64::consts::E).pow(-e!(r));
    let g = SqMatrix([
        [a.clone(), e!(0), e!(0), e!(0)],
        [e!(0), -a, e!(0), e!(0)],
        [e!(0), e!(0), e!(r).pow(2), e!(0)],
        [e!(0), e!(0), e!(0), e!(r).pow(2)],
    ]);

    let x = [1.0];

    println!("{}", sys.strmat(sys.evalmat(g.clone(), x)));
    println!("{}", sys.strmat(sys.evalmat(g.inv(), x)));
}
