use crate::C;
use num_traits::{AsPrimitive, Float};
use std::{iter::Iterator, ops::Range};

/// Returnes the EM wavelength for the provided `freq_hz`.
pub fn freq_to_wavelen<T>(freq_hz: T) -> T
where
    T: Float + 'static,
    usize: AsPrimitive<T>,
{
    C.as_() / freq_hz
}

/// Returns the nth fresnel `zone` thickness at distance `d_m` meters
/// from the transmitter out of `total_d_m` meters.
#[inline]
pub fn fresnel<T>(zone: T, wavelen: T, d_m: T, total_d_m: T) -> T
where
    T: Float,
{
    let d1 = d_m;
    let d2 = total_d_m - d1;
    (zone * wavelen * d1 * d2 / total_d_m).sqrt()
}

/// Represents the lower nth fresnel zone of a radio link.
#[derive(Debug)]
pub struct FresnelZone<T> {
    /// Which fresnel zone we're interested in.
    zone: u8,
    wavelength: T,
    distance_m: T,
}

impl<T> FresnelZone<T> {
    /// Returns a new `FesnelZone` object.
    pub fn new(zone: u8, f_hz: T, distance_m: T) -> Self
    where
        T: Float + 'static,
        usize: AsPrimitive<T>,
    {
        let wavelength = C.as_() / f_hz;
        Self {
            zone,
            wavelength,
            distance_m,
        }
    }

    /// Returns a new `FresnelZoneIter` of length `len`.
    #[allow(clippy::iter_not_returning_iterator)]
    pub fn iter(&self, len: usize) -> FresnelZoneIter<T>
    where
        T: Copy + 'static,
        u8: AsPrimitive<T>,
    {
        FresnelZoneIter {
            zone: self.zone.as_(),
            wavelength: self.wavelength,
            distance_m: self.distance_m,
            range: 0..len,
        }
    }
}

/// An iterator for a fresnel zone the specified frequency.
#[derive(Debug)]
pub struct FresnelZoneIter<T> {
    /// Which fresnel zone we're interested in.
    zone: T,
    wavelength: T,
    range: Range<usize>,
    distance_m: T,
}

impl<T> Iterator for FresnelZoneIter<T>
where
    T: Float + 'static,
    usize: AsPrimitive<T>,
{
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        self.range.next().map(|n| {
            let d1 = self.distance_m * (n.as_() / (self.range.end - 1).as_());
            let d2 = self.distance_m - d1;
            (self.zone * self.wavelength * d1 * d2 / (self.distance_m)).sqrt()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::FresnelZone;

    #[test]
    fn test_zero_len_fresnel_zone_iter() {
        let mut iter = FresnelZone::new(1, 1.0, 10e3).iter(0);
        assert!(iter.next().is_none());
    }

    #[test]
    fn test_1st_fresnel_zone() {
        let mut iter = FresnelZone::new(1, 900e6, 1e3).iter(3);
        assert_eq!(iter.next(), Some(0.0));
        assert_eq!(iter.next(), Some(9.125_551_094_469_735));
        assert_eq!(iter.next(), Some(0.0));
    }

    #[test]
    fn test_2nd_fresnel_zone() {
        let mut iter = FresnelZone::new(2, 900e6, 1e3).iter(3);
        assert_eq!(iter.next(), Some(0.0));
        assert_eq!(iter.next(), Some(12.905_478_121_927_74));
        assert_eq!(iter.next(), Some(0.0));
    }

    #[test]
    fn test_3rd_fresnel_zone() {
        let mut iter = FresnelZone::new(3, 900e6, 1e3).iter(3);
        assert_eq!(iter.next(), Some(0.0));
        assert_eq!(iter.next(), Some(15.805_918_142_687_355));
        assert_eq!(iter.next(), Some(0.0));
    }
}
