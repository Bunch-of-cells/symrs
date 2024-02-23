use symrs::{Expressable, Expression, SqMatrix, System, Var};

fn main() {
    let mut sys = System::default();
    let x = sys.symbols::<2>("r θ").unwrap();
    let g = SqMatrix([
        [1.0.ex(), 0.0.ex()],
        [0.0.ex(), 1.0.ex()]
    ]);
    println!("Metric Tensor:\n{}", sys.strmat(&g));
    let gamma = christoffel(g.clone(), g.inverse(), x);
    let curvature = ricci_tensor(gamma, x);
    print_curvature(curvature, &sys);
}

fn print_curvature<const N: usize>(curvature: Curvature<N>, sys: &System) {
    let mut f = String::new();
    for x in curvature.iter() {
        f += "[";
        for x in x.iter() {
            f.push_str(&format!("{:5}", sys.str(x)));
        }
        f += "]\n";
    }
    println!("Ricci Tensor:\n{f}");
}

type Christoffel<const N: usize> = [[[Expression; N]; N]; N];
fn christoffel<const N: usize>(g: SqMatrix<N>, g_inv: SqMatrix<N>, x: [Var; N]) -> Christoffel<N> {
    let mut gamma =
        std::array::from_fn(|_| std::array::from_fn(|_| std::array::from_fn(|_| 0.0.ex())));
    for (m, c) in gamma.iter_mut().enumerate() {
        for (n, c) in c.iter_mut().enumerate() {
            for (r, c) in c.iter_mut().enumerate() {
                let mut sum = 0.0.ex();
                for s in 0..N {
                    sum = sum
                        + g_inv[r][s].clone()
                            * (g[m][s].diff(x[n]) + g[n][s].diff(x[m]) - g[m][n].diff(x[s]));
                }
                *c = sum * 0.5;
            }
        }
    }
    gamma
}

type Curvature<const N: usize> = [[Expression; N]; N];
fn ricci_tensor<const N: usize>(gamma: Christoffel<N>, x: [Var; N]) -> Curvature<N> {
    let mut r = std::array::from_fn(|_| std::array::from_fn(|_| 0.0.ex()));
    for (i, c) in r.iter_mut().enumerate() {
        for (j, c) in c.iter_mut().enumerate() {
            let mut sum = 0.0.ex();
            for (a, &x) in x.iter().enumerate() {
                sum = sum + gamma[i][j][a].diff(x);
            }
            for (a, &x) in x.iter().enumerate() {
                sum = sum + gamma[i][j][a].diff(x);
            }
            for a in 0..N {
                for b in 0..N {
                    sum = sum + gamma[a][b][a].clone() * gamma[i][j][b].clone()
                        - gamma[i][b][a].clone() * gamma[a][j][b].clone();
                }
            }
            *c = sum.simplify();
        }
    }
    r
}
