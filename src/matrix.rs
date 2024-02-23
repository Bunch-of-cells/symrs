use std::ops::{Add, Index, IndexMut, Mul, Sub};

use crate::{Expressable, Expression, Var};

#[derive(Debug, Clone)]
pub struct SqMatrix<const N: usize>(pub [[Expression; N]; N]);

impl SqMatrix<2> {
    pub fn det(&self) -> Expression {
        self.0[0][0].clone() * self.0[1][1].clone() - self.0[0][1].clone() * self.0[1][0].clone()
    }

    pub fn inverse(&self) -> Self {
        let det = self.det();
        SqMatrix([
            [
                self.0[1][1].clone() / det.clone(),
                -self.0[0][1].clone() / det.clone(),
            ],
            [
                -self.0[1][0].clone() / det.clone(),
                self.0[0][0].clone() / det,
            ],
        ])
    }
}

impl SqMatrix<3> {
    pub fn det(&self) -> Expression {
        let m = &self.0;
        let a = m[1][1].clone() * m[2][2].clone() - m[1][2].clone() * m[2][1].clone();
        let nb = m[2][0].clone() * m[2][2].clone() - m[1][2].clone() * m[2][0].clone();
        let c = m[2][0].clone() * m[2][1].clone() - m[1][1].clone() * m[2][0].clone();
        m[0][0].clone() * a - m[0][1].clone() * nb + m[0][2].clone() * c
    }

    pub fn inverse(&self) -> Self {
        let det = self.det();
        let m = &self.0;
        let a = m[1][1].clone() * m[2][2].clone() - m[1][2].clone() * m[2][1].clone();
        let nb = m[2][0].clone() * m[2][2].clone() - m[1][2].clone() * m[2][0].clone();
        let c = m[2][0].clone() * m[2][1].clone() - m[1][1].clone() * m[2][0].clone();
        let nd = m[0][1].clone() * m[2][2].clone() - m[0][2].clone() * m[2][1].clone();
        let e = m[0][0].clone() * m[2][2].clone() - m[0][2].clone() * m[2][1].clone();
        let nf = m[0][0].clone() * m[2][1].clone() - m[0][1].clone() * m[2][0].clone();
        let g = m[0][1].clone() * m[1][2].clone() - m[0][2].clone() * m[1][1].clone();
        let nh = m[0][0].clone() * m[1][2].clone() - m[0][2].clone() * m[1][0].clone();
        let i = m[0][0].clone() * m[1][1].clone() - m[0][1].clone() * m[1][0].clone();
        SqMatrix([
            [a / det.clone(), -nd / det.clone(), g / det.clone()],
            [-nb / det.clone(), e / det.clone(), -nh / det.clone()],
            [c / det.clone(), -nf / det.clone(), i / det.clone()],
        ])
    }
}

impl<const N: usize> SqMatrix<N> {
    pub fn zeroes() -> Self {
        Self(std::array::from_fn(|_| std::array::from_fn(|_| 0.0.ex())))
    }

    pub fn trace(&self) -> Expression {
        let mut sum = 0.0.ex();
        for i in 0..N {
            sum = sum + self.0[i][i].clone();
        }
        sum
    }

    pub fn diff(mut self, x: Var) -> Self {
        for i in 0..N {
            for j in 0..N {
                self.0[i][j] = self.0[i][j].diff(x);
            }
        }
        self
    }

    pub fn simplify(mut self) -> Self {
        for i in 0..N {
            for j in 0..N {
                self.0[i][j] = self.0[i][j].simplify();
            }
        }
        self
    }
}

impl<const N: usize> Mul for SqMatrix<N> {
    type Output = SqMatrix<N>;
    fn mul(self, rhs: Self) -> Self::Output {
        let mut c = std::array::from_fn(|_| std::array::from_fn(|_| 0.0.ex()));
        for (i, c) in c.iter_mut().enumerate() {
            for (j, c) in c.iter_mut().enumerate() {
                let mut sum = 0.0.ex();
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
        let mut c = std::array::from_fn(|_| std::array::from_fn(|_| 0.0.ex()));
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
        let mut c = std::array::from_fn(|_| std::array::from_fn(|_| 0.0.ex()));
        for (i, c) in c.iter_mut().enumerate() {
            for (j, c) in c.iter_mut().enumerate() {
                *c = self.0[i][j].clone() - rhs.0[i][j].clone();
            }
        }
        SqMatrix(c)
    }
}

impl<const N: usize, T: Expressable> Mul<T> for SqMatrix<N> {
    type Output = SqMatrix<N>;
    fn mul(self, rhs: T) -> Self::Output {
        let mut c = std::array::from_fn(|_| std::array::from_fn(|_| 0.0.ex()));
        for (i, c) in c.iter_mut().enumerate() {
            for (j, c) in c.iter_mut().enumerate() {
                *c = self.0[i][j].clone() * rhs.clone();
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
