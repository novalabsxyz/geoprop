//! These routines are taken from the [geo] crate, modified to better
//! fit our use-case.
//!
//! [geo](https://github.com/georust/geo/blob/eb0cd98f3ccfa226631af23d94d66d214ea66488/geo/src/algorithm/haversine_intermediate.rs)

use crate::constants::MEAN_EARTH_RADIUS;
use geo::{CoordFloat, Point};
use num_traits::{AsPrimitive, FromPrimitive};

pub struct HaversineIter<T: CoordFloat = f32> {
    params: HaversineParams<T>,
    step_size_m: T,
    total_points: T,
    current_point: T,
    inverse: T,
}

impl<T: CoordFloat> HaversineIter<T> {
    pub fn new(start: Point<T>, max_step_size: T, end: Point<T>) -> Self
    where
        T: FromPrimitive + AsPrimitive<usize>,
    {
        let params = get_params(&start, &end);
        let HaversineParams { d, .. } = params;
        let total_distance = d * T::from(MEAN_EARTH_RADIUS).unwrap();
        let number_of_points = (total_distance / max_step_size).ceil();
        let step_size_m = total_distance / number_of_points;

        Self {
            params,
            step_size_m,
            total_points: number_of_points + T::one(),
            current_point: T::zero(),
            inverse: T::one() / number_of_points,
        }
    }

    #[allow(dead_code)]
    pub fn step_size_m(&self) -> T {
        self.step_size_m
    }
}

impl<T: CoordFloat + AsPrimitive<usize>> Iterator for HaversineIter<T> {
    type Item = Point<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.current_point < self.total_points {
            let factor = self.current_point * self.inverse;
            self.current_point = self.current_point + T::one();
            Some(get_point(&self.params, factor))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.total_points.as_() - self.current_point.as_();
        (remaining, Some(remaining))
    }
}

impl<T: CoordFloat + AsPrimitive<usize>> ExactSizeIterator for HaversineIter<T> {
    fn len(&self) -> usize {
        self.total_points.as_() - self.current_point.as_()
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
    T: CoordFloat,
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

    let lat = z.atan2(x.hypot(y));
    let lon = y.atan2(x);

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

#[cfg(test)]
mod tests {
    use super::HaversineIter;
    use approx::assert_relative_eq;
    use geo::point;

    #[test]
    fn test_haversine_iter() {
        let start = point!(x: -0.5, y: -0.5);
        let end = point!(x: 0.5, y: 0.5);
        let step_size_m = 17_472.510_284_442_324;
        let haversine = HaversineIter::new(start, step_size_m, end);
        assert_eq!(haversine.len(), 10);
        assert_relative_eq!(haversine.step_size_m(), step_size_m);
        let points = haversine.collect::<Vec<_>>();
        let expected = vec![
            point!(x: -0.5, y: -0.5),
            point!(x: -0.388_884_988_799_152_34, y: -0.388_890_838_895_255_3),
            point!(x: -0.277_772_902_687_608_4, y: -0.277_780_215_266_485_2),
            point!(x: -0.166_662_905_894_136_8, y: -0.166_668_547_005_197_93),
            point!(x: -0.055_554_162_678_936_12, y: -0.055_556_251_975_400_386),
            point!(x: 0.055_554_162_678_936_12, y: 0.055_556_251_975_400_386),
            point!(x: 0.166_662_905_894_136_7, y: 0.166_668_547_005_197_84),
            point!(x: 0.277_772_902_687_608_24, y: 0.277_780_215_266_485_1),
            point!(x: 0.388_884_988_799_152_3, y: 0.388_890_838_895_255_2),
            point!(x: 0.5, y: 0.5),
        ];
        assert_eq!(points, expected);
    }
}
