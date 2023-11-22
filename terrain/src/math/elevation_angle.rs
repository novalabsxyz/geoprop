use num_traits::{Float, FloatConst};

/// Returns the up/down angle (in radians) from a to b.
pub fn elevation_angle<T>(start_elev_m: T, distance_m: T, end_elev_m: T, earth_radius: T) -> T
where
    T: Float + FloatConst,
{
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

#[cfg(test)]
mod tests {
    use super::elevation_angle;
    use crate::constants::MEAN_EARTH_RADIUS;
    use approx::assert_relative_eq;

    #[test]
    fn test_elevation_angle() {
        assert_relative_eq!(
            0.100_167_342_359_641_42,
            elevation_angle(1.0, 1.0, 1.1, MEAN_EARTH_RADIUS),
            epsilon = f64::EPSILON
        );
    }
}
