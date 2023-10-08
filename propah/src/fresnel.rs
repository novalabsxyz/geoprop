use num_traits::{AsPrimitive, Float};
use std::{iter::Iterator, ops::Range};

/// Speed of light in m/s
const C: usize = 299_792_458;

/// Represents the lower nth fresnel zone of a radio link.
#[derive(Debug)]
pub struct FresnelZone<T> {
    /// Which fresnel zone we're interested in.
    zone: u8,
    wavelength: T,
    distance_m: T,
}

impl<T> FresnelZone<T> {
    /// Returns a new FesnelZone object.
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

    /// Returns a new FresnelZoneIter of length `len`.
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
        assert_eq!(iter.next(), Some(9.125551094469735));
        assert_eq!(iter.next(), Some(0.0));
    }

    #[test]
    fn test_2nd_fresnel_zone() {
        let mut iter = FresnelZone::new(2, 900e6, 1e3).iter(3);
        assert_eq!(iter.next(), Some(0.0));
        assert_eq!(iter.next(), Some(12.90547812192774));
        assert_eq!(iter.next(), Some(0.0));
    }

    #[test]
    fn test_3rd_fresnel_zone() {
        let mut iter = FresnelZone::new(3, 900e6, 1e3).iter(3);
        assert_eq!(iter.next(), Some(0.0));
        assert_eq!(iter.next(), Some(15.805918142687355));
        assert_eq!(iter.next(), Some(0.0));
    }
}
