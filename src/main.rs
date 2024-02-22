use symrs::System;

fn main() {
    let mut sys = System::default();
    let [x, _y, _z] = sys.symbols("x y z").unwrap();
    let a = x + x;
    println!("{}", sys.str(a.clone()));
    let w = a.d(x);
    println!("{}", sys.str(w.clone()));
}
