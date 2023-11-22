use num_traits::{Float, FromPrimitive};

pub fn linspace<T>(y_start: T, y_end: T, n: usize) -> impl Iterator<Item = T>
where
    T: Float + FromPrimitive,
{
    let dy = (y_end - y_start) / T::from(n - 1).unwrap();
    (0..n).map(move |x| y_start + T::from(x).unwrap() * dy)
}
