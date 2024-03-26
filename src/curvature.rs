use symrs::{e, Expression, SqMatrix, System, Var};

pub fn print_curvature<const N: usize>(curvature: RicciCurvature<N>, sys: &System) {
    let mut f = String::new();
    for x in curvature.into_iter() {
        f += "[";
        for x in x.into_iter() {
            f.push_str(&format!("{:5}", sys.str(x.simplify())));
        }
        f += "]\n";
    }
    println!("Ricci Tensor:\n{f}");
}

pub type Christoffel<const N: usize> = [[[Expression; N]; N]; N];
pub fn christoffel<const N: usize>(
    g: SqMatrix<N>,
    g_inv: SqMatrix<N>,
    x: [Var; N],
) -> Christoffel<N> {
    let mut gamma =
        std::array::from_fn(|_| std::array::from_fn(|_| std::array::from_fn(|_| 0.into())));
    for (m, c) in gamma.iter_mut().enumerate() {
        for (n, c) in c.iter_mut().enumerate() {
            for (r, c) in c.iter_mut().enumerate() {
                let mut sum = e!(0);
                for s in 0..N {
                    sum = sum
                        + g_inv[r][s].clone()
                            * (g[m][s].clone().diff(x[n]) + g[n][s].clone().diff(x[m])
                                - g[m][n].clone().diff(x[s]));
                }
                *c = sum * 0.5.into();
            }
        }
    }
    gamma
}

pub type RiemannCurvature<const N: usize> = [[[[Expression; N]; N]; N]; N];
pub fn riemann_tensor<const N: usize>(gamma: &Christoffel<N>, x: [Var; N]) -> RiemannCurvature<N> {
    let mut r = std::array::from_fn(|_| {
        std::array::from_fn(|_| std::array::from_fn(|_| std::array::from_fn(|_| 0.into())))
    });
    for (i, c) in r.iter_mut().enumerate() {
        for (j, c) in c.iter_mut().enumerate() {
            for (k, c) in c.iter_mut().enumerate() {
                for (l, c) in c.iter_mut().enumerate() {
                    let mut sum =
                        gamma[i][j][l].clone().diff(x[k]) - gamma[i][j][k].clone().diff(x[l]);
                    for m in 0..N {
                        sum = sum + gamma[i][m][k].clone() * gamma[m][j][l].clone()
                            - gamma[i][m][l].clone() * gamma[m][j][k].clone();
                    }
                    *c = sum;
                }
            }
        }
    }
    r
}

pub type RicciCurvature<const N: usize> = [[Expression; N]; N];
pub fn ricci_tensor<const N: usize>(riemann_tensor: &RiemannCurvature<N>) -> RicciCurvature<N> {
    let mut r = std::array::from_fn(|_| std::array::from_fn(|_| 0.into()));
    for (i, c) in r.iter_mut().enumerate() {
        for (j, c) in c.iter_mut().enumerate() {
            #[allow(clippy::needless_range_loop)]
            for k in 0..N {
                *c = riemann_tensor[k][i][k][j].clone();
            }
        }
    }
    r
}

pub fn scalar_curvature<const N: usize>(
    ricci_tensor: &RicciCurvature<N>,
    g_inv: SqMatrix<N>,
) -> Expression {
    let mut r = 0.into();
    for m in 0..N {
        for n in 0..N {
            r = r + g_inv[m][n].clone() * ricci_tensor[m][n].clone()
        }
    }
    r
}
