use num_traits::{AsPrimitive, Float};
use std::{iter::Iterator, num::NonZeroU8, ops::Range};

/// Speed of light in m/s
const C: usize = 299_792_458;

/// Represents the lower nth fresnel zone of a radio link.
#[derive(Debug)]
pub struct FresnelZone<T> {
    /// Which fresnel zone we're interested in.
    _zone: NonZeroU8,
    wavelength: T,
    _distance_m: T,
}

impl<T> FresnelZone<T> {
    /// Returns a new FesnelZone object.
    pub fn new(zone: NonZeroU8, f_hz: T, distance_m: T) -> Self
    where
        T: Float + 'static,
        usize: AsPrimitive<T>,
    {
        let wavelength = C.as_() / f_hz;
        Self {
            _zone: zone,
            wavelength,
            _distance_m: distance_m,
        }
    }

    /// Returns a new FresnelZoneIter of length `len`.
    pub fn iter(&self, len: usize) -> FresnelZoneIter<T>
    where
        T: Copy,
    {
        FresnelZoneIter {
            _zone: self._zone,
            wavelength: self.wavelength,
            range: 0..len,
        }
    }
}

/// An iterator for a fresnel zone the specified frequency.
#[derive(Debug)]
pub struct FresnelZoneIter<T> {
    /// Which fresnel zone we're interested in.
    _zone: NonZeroU8,
    wavelength: T,
    range: Range<usize>,
}

impl<T: Float + 'static> Iterator for FresnelZoneIter<T>
where
    usize: AsPrimitive<T>,
{
    type Item = T;
    fn next(&mut self) -> Option<T> {
        self.range.next().map(|n| n.as_() * self.wavelength)
    }
}

#[cfg(test)]
mod tests {
    use super::{FresnelZone, NonZeroU8};

    #[test]
    fn test_fresnel_zone_iter() {
        let iter = FresnelZone::new(NonZeroU8::new(1).unwrap(), 1.0, 10e3).iter(5);
        let expected = &[0., 1., 2., 3., 4.];
        assert!(iter.zip(expected.iter()).all(|(l, &e)| {
            debug_assert_eq!(l, e);
            l == e
        }));
    }

    #[test]
    fn test_zero_len_fresnel_zone_iter() {
        let mut iter = FresnelZone::new(NonZeroU8::new(1).unwrap(), 1.0, 10e3).iter(0);
        assert!(iter.next().is_none());
    }
}
