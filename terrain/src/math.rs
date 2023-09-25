//! These routines are taken from the [geo] crate, modified to better
//! fit our use-case.
//!
//! [geo](https://github.com/georust/geo/blob/eb0cd98f3ccfa226631af23d94d66d214ea66488/geo/src/algorithm/haversine_intermediate.rs)

use crate::constants::MEAN_EARTH_RADIUS;
use geo::{CoordFloat, Point};
use num_traits::FromPrimitive;

pub(crate) struct HaversineIter<T: CoordFloat + FromPrimitive = f32> {
    start: Option<Point<T>>,
    end: Option<Point<T>>,
    params: HaversineParams<T>,
    interval: T,
    current_step: T,
}

impl<T: CoordFloat + FromPrimitive> HaversineIter<T> {
    pub fn new(start: Point<T>, max_step_size: T, end: Point<T>) -> Self {
        let params = get_params(&start, &end);
        let HaversineParams { d, .. } = params;
        let total_distance = d * T::from(MEAN_EARTH_RADIUS).unwrap();
        let number_of_points = (total_distance / max_step_size).ceil();
        let interval = T::one() / number_of_points;
        let current_step = T::zero();

        Self {
            start: Some(start),
            end: Some(end),
            params,
            interval,
            current_step,
        }
    }
}

impl<T: CoordFloat + FromPrimitive + Atan2> Iterator for HaversineIter<T> {
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
struct HaversineParams<T: num_traits::Float + FromPrimitive> {
    d: T,
    n: T,
    o: T,
    p: T,
    q: T,
    r: T,
    s: T,
}

#[allow(clippy::many_single_char_names)]
fn get_point<T: CoordFloat + FromPrimitive + Atan2>(params: &HaversineParams<T>, f: T) -> Point<T> {
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
fn get_params<T: CoordFloat + FromPrimitive>(p1: &Point<T>, p2: &Point<T>) -> HaversineParams<T> {
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
