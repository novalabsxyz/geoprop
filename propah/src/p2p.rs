use crate::{
    error::PropahError,
    fresnel::{freq_to_wavelen, fresnel},
};
use geo::{Coord, CoordFloat, Point};
use num_traits::{AsPrimitive, Float, FloatConst, FromPrimitive};
use terrain::{Profile, Tiles};

/// Point to point propogation estimate.
#[derive(Debug, Clone)]
pub struct Point2Point<C: CoordFloat> {
    /// Incremental path distance for all following vectors.
    pub distances_m: Vec<C>,

    /// Location of step along the great circle route from `start` to
    /// `end`.
    pub great_circle: Vec<Point<C>>,

    /// Elevation at each step along the great circle route from
    /// `start` to `end`.
    pub terrain_elev_m: Vec<C>,

    /// A straight line from `start` to `end`.
    pub los_elev_m: Vec<C>,

    /// Fresnel zone thickness from `start` to `end`.
    pub fresnel_zone_m: Vec<C>,
}

impl<C> Point2Point<C>
where
    C: CoordFloat,
{
    pub fn builder() -> Point2PointBuilder<C> {
        Point2PointBuilder {
            freq_hz: None,
            start: None,
            max_step_m: None,
            end: None,
            start_alt_m: C::zero(),
            end_alt_m: C::zero(),
            earth_curve: false,
            normalize: false,
        }
    }
}

pub struct Point2PointBuilder<C: CoordFloat = f32> {
    /// Transmitter frequency (required).
    freq_hz: Option<C>,

    /// Start point of the path (required).
    start: Option<Coord<C>>,

    /// Maximum distance between points (required).
    max_step_m: Option<C>,

    /// End point of the path (required).
    end: Option<Coord<C>>,

    /// Starting altitude above ground (meters, defaults to 0).
    start_alt_m: C,

    /// Starting altitude above ground (meters, defaults to 0).
    end_alt_m: C,

    /// Add earth curvature (defaults to false).
    earth_curve: bool,

    /// Place virtual earth curve as the highest and center point of
    /// the output (defaults to false; has no effect if `earth_curve`
    /// is `false`).
    normalize: bool,
}

impl<C> Point2PointBuilder<C>
where
    C: CoordFloat + FromPrimitive,
    f64: From<C>,
{
    /// Frequency of signal (Hz, required).
    #[must_use]
    pub fn freq(mut self, freq_hz: C) -> Self {
        self.freq_hz = Some(freq_hz);
        self
    }

    /// Start point of the path (required).
    #[must_use]
    pub fn start(mut self, coord: Coord<C>) -> Self {
        self.start = Some(coord);
        self
    }

    /// Starting altitude above ground (meters, defaults to 0).
    #[must_use]
    pub fn start_alt(mut self, meters: C) -> Self {
        self.start_alt_m = meters;
        self
    }

    /// Maximum distance between points (required).
    #[must_use]
    pub fn max_step(mut self, meters: C) -> Self {
        self.max_step_m = Some(meters);
        self
    }

    /// End point of the path (required).
    #[must_use]
    pub fn end(mut self, coord: Coord<C>) -> Self {
        self.end = Some(coord);
        self
    }

    /// Starting altitude above ground (meters, defaults to 0).
    #[must_use]
    pub fn end_alt(mut self, meters: C) -> Self {
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

    pub fn build(&self, tiles: &Tiles) -> Result<Point2Point<C>, PropahError>
    where
        C: FloatConst + Float + 'static,
        usize: AsPrimitive<C>,
        C: AsPrimitive<usize>,
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
        let fresnel_zone_m = distances_m
            .iter()
            .map(|&d1| fresnel(fresnel_zone_1, wavelen, d1, total_distance_m))
            .collect();

        Ok(Point2Point {
            distances_m,
            great_circle,
            terrain_elev_m,
            los_elev_m,
            fresnel_zone_m,
        })
    }
}
