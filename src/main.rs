use symrs::{Expressable, SqMatrix, System};

fn main() {
    let mut sys = System::default();
    let [x] = sys.symbols("x").unwrap();
    let g = SqMatrix([
        [x.ex(), 0.0.ex()],
        [0.0.ex(), x.ex()],
    ]);
    println!("{}", sys.strmat(&g));
    println!("{}", sys.strmat(&g.diff(x)));
    let a = 2.0.ex() * x;
    let w = a.diff(x);
    println!("{}", sys.str(&w.simplify()));
}
