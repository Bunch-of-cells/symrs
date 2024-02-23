use symrs::System;

fn main() {
    let mut sys = System::default();
    let [x, y, z] = sys.symbols("x y z").unwrap();
    let a = x * 2.0 + y;
    println!("{}", sys.str(a.simplify()));
    let w = a.diff(z).simplify();
    println!("{}", sys.str(w.clone()));
}
