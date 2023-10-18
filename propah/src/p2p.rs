use crate::geo::{Coord, CoordFloat, Point};
use crate::{
    error::PropahError,
    fresnel::{freq_to_wavelen, fresnel},
};
use num_traits::{AsPrimitive, Float, FloatConst, FromPrimitive};
use terrain::{constants::MEAN_EARTH_RADIUS, Profile, Tiles};

/// Point to point propogation estimate.
#[derive(Debug, Clone)]
pub struct Point2Point<T: CoordFloat> {
    /// Incremental path distance for all following vectors.
    pub distances_m: Box<[T]>,

    /// Location of step along the great circle route from `start` to
    /// `end`.
    pub great_circle: Box<[Point<T>]>,

    /// Elevation at each step along the great circle route from
    /// `start` to `end`.
    pub terrain_elev_m: Box<[T]>,

    /// A straight line from `start` to `end`.
    pub los_elev_m: Box<[T]>,

    /// Fresnel zone thickness from `start` to `end`.
    ///
    /// Due to its usefullness, this is the lower envelope of the
    /// fresnel zone.
    pub lower_fresnel_zone_m: Box<[T]>,
}

impl<T> Point2Point<T>
where
    T: CoordFloat,
{
    /// Returns a `Point2Point` builder.
    ///
    /// See [`Point2PointBuilder`] docs.
    pub fn builder() -> Point2PointBuilder<T> {
        Point2PointBuilder {
            freq_hz: None,
            start: None,
            max_step_m: None,
            end: None,
            start_alt_m: T::zero(),
            end_alt_m: T::zero(),
            earth_curve: false,
            normalize: false,
            earth_radius: T::from(MEAN_EARTH_RADIUS).unwrap(),
        }
    }

    /// Returns how may points are in the path.
    pub fn len(&self) -> usize {
        // Point2Point can never be empty.
        #![allow(clippy::len_without_is_empty)]
        // All member lengths are the same and enforced at
        // construction, so any will do.
        self.distances_m.len()
    }

    /// Returns an iterator over the elements of the upper fresnel
    /// zone.
    pub fn upper_fresnel_iter(&self) -> impl Iterator<Item = T> + '_
    where
        T: Float,
    {
        self.lower_fresnel_zone_m
            .iter()
            .zip(self.los_elev_m.iter())
            .map(|(&bottom, &los)| los + los - bottom)
    }
}

pub struct Point2PointBuilder<T: CoordFloat = f32> {
    /// Transmitter frequency (required).
    freq_hz: Option<T>,

    /// Start point of the path (required).
    start: Option<Coord<T>>,

    /// Maximum distance between points (required).
    max_step_m: Option<T>,

    /// End point of the path (required).
    end: Option<Coord<T>>,

    /// Starting altitude above ground (meters, defaults to 0).
    start_alt_m: T,

    /// Starting altitude above ground (meters, defaults to 0).
    end_alt_m: T,

    /// Add earth curvature (defaults to false).
    earth_curve: bool,

    /// Place virtual earth curve as the highest and center point of
    /// the output (defaults to false; has no effect if `earth_curve`
    /// is `false`).
    normalize: bool,

    /// Earth radius, defaults to [MEAN_EARTH_RADIUS].
    earth_radius: T,
}

impl<T> Point2PointBuilder<T>
where
    T: CoordFloat + FromPrimitive,
    f64: From<T>,
{
    /// Frequency of signal (Hz, required).
    #[must_use]
    pub fn freq(mut self, freq_hz: T) -> Self {
        self.freq_hz = Some(freq_hz);
        self
    }

    /// Start point of the path (required).
    #[must_use]
    pub fn start(mut self, coord: Coord<T>) -> Self {
        self.start = Some(coord);
        self
    }

    /// Starting altitude above ground (meters, defaults to 0).
    #[must_use]
    pub fn start_alt(mut self, meters: T) -> Self {
        self.start_alt_m = meters;
        self
    }

    /// Maximum distance between points (required).
    #[must_use]
    pub fn max_step(mut self, meters: T) -> Self {
        self.max_step_m = Some(meters);
        self
    }

    /// End point of the path (required).
    #[must_use]
    pub fn end(mut self, coord: Coord<T>) -> Self {
        self.end = Some(coord);
        self
    }

    /// Starting altitude above ground (meters, defaults to 0).
    #[must_use]
    pub fn end_alt(mut self, meters: T) -> Self {
        self.end_alt_m = meters;
        self
    }

    /// Add earth curvature (defaults to false).
    #[must_use]
    pub fn earth_curve(mut self, add_curve: bool) -> Self {
        self.earth_curve = add_curve;
        self
    }

    /// Place virtual earth curve as the highest and center point of
    /// the output (defaults to false; has no effect if `earth_curve`
    /// is `false`).
    #[must_use]
    pub fn normalize(mut self, normalize: bool) -> Self {
        self.normalize = normalize;
        self
    }

    /// Earth radius (meters, defaults to [`MEAN_EARTH_RADIUS`]).
    #[must_use]
    pub fn earth_radius(mut self, earth_radius_m: T) -> Self {
        self.earth_radius = earth_radius_m;
        self
    }

    pub fn build(&self, tiles: &Tiles) -> Result<Point2Point<T>, PropahError>
    where
        T: FloatConst + Float + 'static,
        usize: AsPrimitive<T>,
        T: AsPrimitive<usize>,
    {
        let freq_hz = self.freq_hz.ok_or(PropahError::Builder("freq"))?;
        let start = self.start.ok_or(PropahError::Builder("start"))?;
        let max_step_m = self.max_step_m.ok_or(PropahError::Builder("max_step"))?;
        let end = self.end.ok_or(PropahError::Builder("end"))?;

        let Profile {
            distances_m,
            great_circle,
            terrain_elev_m,
            los_elev_m,
        } = Profile::builder()
            .start(start)
            .start_alt(self.start_alt_m)
            .max_step(max_step_m)
            .end(end)
            .end_alt(self.end_alt_m)
            .earth_curve(self.earth_curve)
            .normalize(self.normalize)
            .build(tiles)?;

        // Unwrap is fine as profiles always have at least two points.
        let total_distance_m = *distances_m.last().unwrap();
        let wavelen = freq_to_wavelen(freq_hz);
        let fresnel_zone_1 = 1.as_();
        let fresnel_zone_m: Box<[T]> = distances_m
            .iter()
            .zip(los_elev_m.iter())
            .map(|(&d1, &los_elev_m)| {
                los_elev_m - fresnel(fresnel_zone_1, wavelen, d1, total_distance_m)
            })
            .collect();

        assert!(
            distances_m.len() == great_circle.len()
                && great_circle.len() == terrain_elev_m.len()
                && terrain_elev_m.len() == los_elev_m.len()
                && los_elev_m.len() == fresnel_zone_m.len(),
            "all vectors in report must have the same length"
        );

        Ok(Point2Point {
            distances_m,
            great_circle,
            terrain_elev_m,
            los_elev_m,
            lower_fresnel_zone_m: fresnel_zone_m,
        })
    }
}
