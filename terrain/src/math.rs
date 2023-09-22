pub trait Atan2 {
    fn atan2(lhs: Self, rhs: Self) -> Self;
}

impl Atan2 for f32 {
    fn atan2(lhs: Self, rhs: Self) -> Self {
        fast_math::atan2(lhs, rhs)
    }
}

impl Atan2 for f64 {
    fn atan2(lhs: Self, rhs: Self) -> Self {
        lhs.atan2(rhs)
    }
}
