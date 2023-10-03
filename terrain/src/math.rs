//! These routines are taken from the [geo] crate, modified to better
//! fit our use-case.
//!
//! [geo](https://github.com/georust/geo/blob/eb0cd98f3ccfa226631af23d94d66d214ea66488/geo/src/algorithm/haversine_intermediate.rs)

use crate::constants::MEAN_EARTH_RADIUS;
use geo::{CoordFloat, Point};
use num_traits::{Float, FloatConst, FromPrimitive};

pub struct HaversineIter<T: CoordFloat = f32> {
    start: Option<Point<T>>,
    end: Option<Point<T>>,
    params: HaversineParams<T>,
    step_size_m: T,
    interval: T,
    current_step: T,
}

impl<T: CoordFloat> HaversineIter<T> {
    pub fn new(start: Point<T>, max_step_size: T, end: Point<T>) -> Self
    where
        T: FromPrimitive,
    {
        let params = get_params(&start, &end);
        let HaversineParams { d, .. } = params;
        let total_distance = d * T::from(MEAN_EARTH_RADIUS).unwrap();
        let number_of_points = (total_distance / max_step_size).ceil();
        let step_size_m = total_distance / number_of_points;
        let interval = T::one() / number_of_points;
        let current_step = T::zero();

        Self {
            start: Some(start),
            end: Some(end),
            params,
            step_size_m,
            interval,
            current_step,
        }
    }

    #[allow(dead_code)]
    pub fn step_size_m(&self) -> T {
        self.step_size_m
    }
}

impl<T: CoordFloat + Atan2> Iterator for HaversineIter<T> {
    type Item = Point<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start.is_some() {
            self.current_step = self.current_step + self.interval;
            self.start.take()
        } else if self.current_step < T::one() {
            let ret = Some(get_point(&self.params, self.current_step));
            self.current_step = self.current_step + self.interval;
            ret
        } else {
            self.end.take()
        }
    }
}

#[allow(clippy::many_single_char_names)]
struct HaversineParams<T> {
    d: T,
    n: T,
    o: T,
    p: T,
    q: T,
    r: T,
    s: T,
}

#[allow(clippy::many_single_char_names)]
fn get_point<T>(params: &HaversineParams<T>, f: T) -> Point<T>
where
    T: CoordFloat + Atan2,
{
    let one = T::one();

    let HaversineParams {
        d,
        n,
        o,
        p,
        q,
        r,
        s,
    } = *params;

    let a = ((one - f) * d).sin() / d.sin();
    let b = (f * d).sin() / d.sin();

    let x = a * n + b * o;
    let y = a * p + b * q;
    let z = a * r + b * s;

    let lat = Atan2::atan2(z, x.hypot(y));
    let lon = Atan2::atan2(y, x);

    Point::new(lon.to_degrees(), lat.to_degrees())
}

#[allow(clippy::many_single_char_names)]
fn get_params<T>(p1: &Point<T>, p2: &Point<T>) -> HaversineParams<T>
where
    T: CoordFloat + FromPrimitive,
{
    let one = T::one();
    let two = one + one;

    let lat1 = p1.y().to_radians();
    let lon1 = p1.x().to_radians();
    let lat2 = p2.y().to_radians();
    let lon2 = p2.x().to_radians();

    let (lat1_sin, lat1_cos) = lat1.sin_cos();
    let (lat2_sin, lat2_cos) = lat2.sin_cos();
    let (lon1_sin, lon1_cos) = lon1.sin_cos();
    let (lon2_sin, lon2_cos) = lon2.sin_cos();

    let m = lat1_cos * lat2_cos;

    let n = lat1_cos * lon1_cos;
    let o = lat2_cos * lon2_cos;
    let p = lat1_cos * lon1_sin;
    let q = lat2_cos * lon2_sin;

    let k = (((lat1 - lat2) / two).sin().powi(2) + m * ((lon1 - lon2) / two).sin().powi(2)).sqrt();

    let d = two * k.asin();

    HaversineParams {
        d,
        n,
        o,
        p,
        q,
        r: lat1_sin,
        s: lat2_sin,
    }
}

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

/// Returns the up/down angle (in radians) from a to b.
pub fn elevation_angle<T>(start_elev_m: T, distance_m: T, end_elev_m: T) -> T
where
    T: Float + FloatConst,
{
    let earth_radius = T::from(MEAN_EARTH_RADIUS).unwrap();
    let a = distance_m;
    let b = start_elev_m + earth_radius;
    let c = end_elev_m + earth_radius;
    let inner = {
        let inner = (a.powi(2) + b.powi(2) - c.powi(2)) / ((T::one() + T::one()) * a * b);
        if inner < -T::one() {
            -T::one()
        } else if inner > T::one() {
            T::one()
        } else {
            inner
        }
    };
    inner.acos() - T::FRAC_PI_2()
}

pub fn linspace<T>(y_start: T, y_end: T, n: usize) -> impl Iterator<Item = T>
where
    T: Float + FromPrimitive,
{
    let dy = (y_end - y_start) / T::from(n - 1).unwrap();
    (0..n).map(move |x| y_start + T::from(x).unwrap() * dy)
}

#[cfg(test)]
mod tests {
    use super::{elevation_angle, HaversineIter};
    use approx::assert_relative_eq;
    use geo::point;

    #[test]
    fn test_haversine_iter() {
        let start = point!(x: -0.5, y: -0.5);
        let end = point!(x: 0.5, y: 0.5);
        let step_size_m = 17472.510284442324;
        let haversine = HaversineIter::new(start, step_size_m, end);
        assert_eq!(haversine.step_size_m(), step_size_m);
        let points = haversine.collect::<Vec<_>>();
        let expected = vec![
            point!(x:-0.5,y:-0.5),
            point!(x:-0.38888498879915234,y:-0.3888908388952553),
            point!(x:-0.2777729026876084,y:-0.2777802152664852),
            point!(x:-0.1666629058941368,y:-0.16666854700519793),
            point!(x:-0.05555416267893612,y:-0.055556251975400386),
            point!(x:0.05555416267893612,y:0.055556251975400386),
            point!(x:0.1666629058941368,y:0.16666854700519793),
            point!(x:0.2777729026876085,y:0.27778021526648533),
            point!(x:0.38888498879915245,y:0.3888908388952555),
            point!(x:0.5,y:0.5),
        ];
        assert_eq!(points, expected);
    }

    #[test]
    fn test_elevation_angle() {
        assert_relative_eq!(
            0.10016734235964142,
            elevation_angle(1.0, 1.0, 1.1),
            epsilon = f64::EPSILON
        );
    }
}
