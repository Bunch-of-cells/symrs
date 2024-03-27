use std::{
    array,
    ops::{Add, Div, Index, IndexMut, Mul, Sub},
};

use crate::{c, e, Expressable, Expression, Var, ONE, ZERO};

#[derive(Debug, Clone)]
pub struct SqMatrix<const N: usize>(pub [[Expression; N]; N]);

impl<const N: usize> SqMatrix<N> {
    pub fn tr(&self) -> Expression {
        let mut sum = ZERO.into();
        for i in 0..N {
            sum = sum + self.0[i][i].clone();
        }
        sum
    }

    pub fn identity() -> Self {
        let mut m: [[Expression; N]; N] = array::from_fn(|_| array::from_fn(|_| ZERO.into()));
        for (i, v) in m.iter_mut().enumerate() {
            v[i] = ONE.into();
        }
        SqMatrix(m)
    }

    pub fn zeroes() -> Self {
        SqMatrix(array::from_fn(|_| array::from_fn(|_| ZERO.into())))
    }

    pub fn diff(&self, x: Var) -> Self {
        let mut new = Self::zeroes();
        for i in 0..N {
            for j in 0..N {
                new.0[i][j] = self.0[i][j].clone().diff(x);
            }
        }
        new
    }

    pub fn simplify(&self) -> Self {
        let mut new = Self::zeroes();
        for i in 0..N {
            for j in 0..N {
                new.0[i][j] = self.0[i][j].clone().simplify();
            }
        }
        new
    }

    pub fn pow(&self, n: usize) -> Self {
        (0..n - 1)
            .map(|_| self.clone())
            .fold(self.clone(), Mul::mul)
    }

    pub fn transpose(&self) -> Self {
        let mut trans = Self::zeroes();
        for i in 0..N {
            for j in 0..N {
                trans[i][j] = self[j][i].clone();
            }
        }
        trans
    }

    fn get_cofactor(&self, temp: &mut Self, p: usize, q: usize, n: usize) {
        let mut i = 0;
        let mut j = 0;
        for row in 0..n {
            for col in 0..n {
                if row != p && col != q {
                    temp[i][j] = self[row][col].clone();
                    j += 1;
                    if j == n - 1 {
                        j = 0;
                        i += 1;
                    }
                }
            }
        }
    }

    fn determinant(&self, n: usize) -> Expression {
        match n {
            0 => return ZERO.into(),
            1 => return self.0[0][0].clone(),
            _ => (),
        }
        let mut det = ZERO.into();
        let mut temp = Self::zeroes();
        let mut sgn = ONE;
        for i in 0..n {
            self.get_cofactor(&mut temp, 0, i, n);
            det = det + e!(sgn) * self[0][i].clone() * temp.determinant(n - 1);
            sgn = -sgn;
        }
        det
    }

    pub fn det(&self) -> Expression {
        self.determinant(N)
    }

    pub fn adj(&self) -> Self {
        let mut adj = Self::identity();
        match N {
            0 | 1 => return adj,
            _ => (),
        }
        let mut temp = Self::zeroes();
        for i in 0..N {
            for j in 0..N {
                self.get_cofactor(&mut temp, i, j, N);
                let sgn = c!(-2 * ((i + j) % 2) as i32 + 1);
                adj[j][i] = e!(sgn) * temp.determinant(N - 1);
            }
        }
        adj
    }

    pub fn cofactor(&self) -> Self {
        self.adj().transpose()
    }

    pub fn inv(&self) -> Self {
        self.adj() / self.det()
    }
}

impl<const N: usize> Mul for SqMatrix<N> {
    type Output = SqMatrix<N>;
    fn mul(self, rhs: Self) -> Self::Output {
        let mut c = std::array::from_fn(|_| std::array::from_fn(|_| ZERO.into()));
        for (i, c) in c.iter_mut().enumerate() {
            for (j, c) in c.iter_mut().enumerate() {
                let mut sum = ZERO.into();
                for k in 0..N {
                    sum = sum + self.0[i][k].clone() * rhs.0[k][j].clone();
                }
                *c = sum;
            }
        }
        SqMatrix(c)
    }
}

impl<const N: usize> Add for SqMatrix<N> {
    type Output = SqMatrix<N>;
    fn add(self, rhs: Self) -> Self::Output {
        let mut c = std::array::from_fn(|_| std::array::from_fn(|_| ZERO.into()));
        for (i, c) in c.iter_mut().enumerate() {
            for (j, c) in c.iter_mut().enumerate() {
                *c = self.0[i][j].clone() + rhs.0[i][j].clone();
            }
        }
        SqMatrix(c)
    }
}

impl<const N: usize> Sub for SqMatrix<N> {
    type Output = SqMatrix<N>;
    fn sub(self, rhs: Self) -> Self::Output {
        let mut c = std::array::from_fn(|_| std::array::from_fn(|_| ZERO.into()));
        for (i, c) in c.iter_mut().enumerate() {
            for (j, c) in c.iter_mut().enumerate() {
                *c = self.0[i][j].clone() - rhs.0[i][j].clone();
            }
        }
        SqMatrix(c)
    }
}

impl<const N: usize, T: Clone> Mul<Expressable<T>> for SqMatrix<N>
where
    Expression: From<Expressable<T>>,
{
    type Output = SqMatrix<N>;
    fn mul(self, rhs: Expressable<T>) -> Self::Output {
        let mut c = std::array::from_fn(|_| std::array::from_fn(|_| ZERO.into()));
        for (i, c) in c.iter_mut().enumerate() {
            for (j, c) in c.iter_mut().enumerate() {
                *c = self.0[i][j].clone() * rhs.clone();
            }
        }
        SqMatrix(c)
    }
}

impl<const N: usize, T: Clone> Div<Expressable<T>> for SqMatrix<N>
where
    Expression: From<Expressable<T>>,
{
    type Output = SqMatrix<N>;
    fn div(self, rhs: Expressable<T>) -> Self::Output {
        let mut c = std::array::from_fn(|_| std::array::from_fn(|_| ZERO.into()));
        for (i, c) in c.iter_mut().enumerate() {
            for (j, c) in c.iter_mut().enumerate() {
                *c = self.0[i][j].clone() / rhs.clone();
            }
        }
        SqMatrix(c)
    }
}

impl<const N: usize> Index<usize> for SqMatrix<N> {
    type Output = [Expression; N];
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl<const N: usize> IndexMut<usize> for SqMatrix<N> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}
