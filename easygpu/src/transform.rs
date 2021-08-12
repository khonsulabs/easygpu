use std::ops::{Add, Div, Neg, Sub};

use num_traits::{One, Zero};

pub type ScreenSpace = figures::Pixels;
pub type WorldSpace = figures::Scaled;

#[derive(Clone, Copy, Debug)]
pub struct ScreenTransformation<S>([S; 16]);

impl<S> ScreenTransformation<S>
where
    S: Add<S, Output = S>
        + Sub<S, Output = S>
        + Div<S, Output = S>
        + One
        + Zero
        + Neg<Output = S>
        + Copy,
{
    pub fn ortho(left: S, top: S, right: S, bottom: S, near: S, far: S) -> Self {
        let tx = -((right + left) / (right - left));
        let ty = -((top + bottom) / (top - bottom));
        let tz = -((far + near) / (far - near));

        let zero = S::zero();
        let one = S::one();
        // I never thought I'd write this as real code
        let two = one + one;
        Self([
            // Row one
            two / (right - left),
            zero,
            zero,
            zero,
            // Row two
            zero,
            two / (top - bottom),
            zero,
            zero,
            // Row three
            zero,
            zero,
            -two / (far - near),
            zero,
            // Row four
            tx,
            ty,
            tz,
            one,
        ])
    }
}

impl<S> ScreenTransformation<S>
where
    S: One + Zero + Copy,
{
    #[rustfmt::skip]
    pub fn identity() -> Self {
        let zero = S::zero();
        let one = S::one();
        Self([
            one , zero, zero, zero,
            zero, one , zero, zero,
            zero, zero, one , zero,
            zero, zero, zero, one ,
        ])
    }
}

impl<S> ScreenTransformation<S> {
    #[rustfmt::skip]
    pub fn to_array(self) -> [S; 16] {
        self.0
    }
}
