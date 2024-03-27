use crate::{c, expression::*, Expression};

pub fn sin<T: Clone>(z: T) -> Expression
where
    Expression: From<T>,
{
    Expression::func(Function::Sin, vec![e!(z)])
}

pub fn cos<T: Clone>(z: T) -> Expression
where
    Expression: From<T>,
{
    Expression::func(Function::Cos, vec![e!(z)])
}

pub fn tan<T: Clone>(z: T) -> Expression
where
    Expression: From<T>,
{
    sin(z.clone()) / cos(z)
}

pub fn sec<T: Clone>(z: T) -> Expression
where
    Expression: From<T>,
{
    cos(z).inv()
}

pub fn csc<T: Clone>(z: T) -> Expression
where
    Expression: From<T>,
{
    sin(z).inv()
}

pub fn cot<T: Clone>(z: T) -> Expression
where
    Expression: From<T>,
{
    tan(z).inv()
}

pub fn asin<T: Clone>(z: T) -> Expression
where
    Expression: From<T>,
{
    let z = e!(z);
    ((<Expression as From<Complex64>>::from(ONE) - z.clone().pow::<f64>(2.0)).pow::<f64>(0.5)
        - <Expression as From<Complex64>>::from(c!(i)) * z)
        .ln()
        * <Expression as From<Complex64>>::from(c!(i))
}

pub fn acos<T: Clone>(z: T) -> Expression
where
    Expression: From<T>,
{
    <Expression as From<f64>>::from(std::f64::consts::FRAC_PI_2) - asin(z)
}

pub fn atan<T: Clone>(z: T) -> Expression
where
    Expression: From<T>,
{
    let a = <Expression as From<Complex64>>::from(c!(i)) * e!(z);
    let b = (a.clone() - <Expression as From<f64>>::from(1.0))
        / (a + <Expression as From<f64>>::from(1.0));
    <Expression as From<Complex64>>::from(c!(-0.5 i)) * b.ln()
}

pub fn asec<T: Clone>(z: T) -> Expression
where
    Expression: From<T>,
{
    acos::<Expression>(e!(z).inv())
}

pub fn acsc<T: Clone>(z: T) -> Expression
where
    Expression: From<T>,
{
    asin::<Expression>(e!(z).inv())
}

pub fn acot<T: Clone>(z: T) -> Expression
where
    Expression: From<T>,
{
    atan::<Expression>(e!(z).inv())
}

pub fn sinh<T: Clone>(z: T) -> Expression
where
    Expression: From<T>,
{
    <Expression as From<Complex64>>::from(c!(-1 i))
        * sin::<Expression>(<Expression as From<Complex64>>::from(c!(i)) * e!(z))
}

pub fn cosh<T: Clone>(z: T) -> Expression
where
    Expression: From<T>,
{
    cos::<Expression>(<Expression as From<Complex64>>::from(c!(i)) * e!(z))
}

pub fn tanh<T: Clone>(z: T) -> Expression
where
    Expression: From<T>,
{
    sinh(z.clone()) / cosh(z)
}

pub fn sech<T: Clone>(z: T) -> Expression
where
    Expression: From<T>,
{
    cosh(z).inv()
}

pub fn csch<T: Clone>(z: T) -> Expression
where
    Expression: From<T>,
{
    sinh(z).inv()
}

pub fn coth<T: Clone>(z: T) -> Expression
where
    Expression: From<T>,
{
    tanh(z).inv()
}

pub fn asinh<T: Clone>(z: T) -> Expression
where
    Expression: From<T>,
{
    let z = e!(z);
    ((z.clone().pow::<f64>(2.0) + <Expression as From<Complex64>>::from(ONE)).pow::<f64>(0.5) + z)
        .ln()
}

pub fn acosh<T: Clone>(z: T) -> Expression
where
    Expression: From<T>,
{
    let z = e!(z);
    ((z.clone().pow::<f64>(2.0) - <Expression as From<Complex64>>::from(ONE)).pow::<f64>(0.5) + z)
        .ln()
}

pub fn atanh<T: Clone>(z: T) -> Expression
where
    Expression: From<T>,
{
    let a = e!(z);
    let b = (<Expression as From<f64>>::from(1.0) + a.clone())
        / (<Expression as From<f64>>::from(1.0) - a);
    <Expression as From<f64>>::from(0.5) * b.ln()
}

pub fn asech<T: Clone>(z: T) -> Expression
where
    Expression: From<T>,
{
    acosh::<Expression>(e!(z).inv())
}

pub fn acsch<T: Clone>(z: T) -> Expression
where
    Expression: From<T>,
{
    asinh::<Expression>(e!(z).inv())
}

pub fn acoth<T: Clone>(z: T) -> Expression
where
    Expression: From<T>,
{
    atanh::<Expression>(e!(z).inv())
}
