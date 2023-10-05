use crate::{
    math::{elevation_angle, linspace, HaversineIter},
    TerrainError, Tiles,
};
use geo::{
    algorithm::HaversineDistance,
    geometry::{Coord, Point},
    CoordFloat,
};
use log::debug;
use num_traits::{AsPrimitive, FloatConst, FromPrimitive};

#[derive(Debug, Clone, PartialEq)]
pub struct Profile<C: CoordFloat = f32> {
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
}

impl<C> Profile<C>
where
    C: CoordFloat,
{
    pub fn builder() -> ProfileBuilder<C> {
        ProfileBuilder {
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

pub struct ProfileBuilder<C: CoordFloat = f32> {
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

impl<C> ProfileBuilder<C>
where
    C: CoordFloat + FromPrimitive,
    f64: From<C>,
{
    /// Start point of the path (required).
    pub fn start(mut self, coord: Coord<C>) -> Self {
        self.start = Some(coord);
        self
    }

    /// Starting altitude above ground (meters, defaults to 0).
    pub fn start_alt(mut self, meters: C) -> Self {
        self.start_alt_m = meters;
        self
    }

    /// Maximum distance between points (required).
    pub fn max_step(mut self, meters: C) -> Self {
        self.max_step_m = Some(meters);
        self
    }

    /// End point of the path (required).
    pub fn end(mut self, coord: Coord<C>) -> Self {
        self.end = Some(coord);
        self
    }

    /// Starting altitude above ground (meters, defaults to 0).
    pub fn end_alt(mut self, meters: C) -> Self {
        self.end_alt_m = meters;
        self
    }

    /// Add earth curvature (defaults to false).
    pub fn earth_curve(mut self, add_curve: bool) -> Self {
        self.earth_curve = add_curve;
        self
    }

    /// Place virtual earth curve as the highest and center point of
    /// the output (defaults to false; has no effect if `earth_curve`
    /// is `false`).
    pub fn normalize(mut self, normalize: bool) -> Self {
        self.normalize = normalize;
        self
    }

    pub fn build(&self, tiles: &Tiles) -> Result<Profile<C>, TerrainError>
    where
        C: FloatConst + AsPrimitive<usize>,
    {
        let start = self.start.ok_or(TerrainError::Builder("start"))?;
        let max_step_m = self.max_step_m.ok_or(TerrainError::Builder("max_step"))?;
        let end = self.end.ok_or(TerrainError::Builder("end"))?;

        let start_point = Point::from(start);
        let end_point = Point::from(end);
        let distance_m = start_point.haversine_distance(&end_point);

        let (great_circle, path_runtime) = {
            let now = std::time::Instant::now();
            let great_circle: Vec<Point<C>> =
                HaversineIter::new(Point::from(start), max_step_m, Point::from(end)).collect();
            let runtime = now.elapsed();
            (great_circle, runtime)
        };

        let (mut terrain_elev_m, terrain_runtime) = {
            let mut terrain = Vec::with_capacity(great_circle.len());
            let now = std::time::Instant::now();
            let mut tile = tiles.get(Coord {
                x: start.x.into(),
                y: start.y.into(),
            })?;

            for point in &great_circle {
                let coord = Coord {
                    x: point.0.x.into(),
                    y: point.0.y.into(),
                };
                if let Some(elevation) = tile.get(coord) {
                    terrain.push(C::from(elevation).unwrap());
                } else {
                    tile = tiles.get(coord)?;
                    let elevation = tile.get_unchecked(coord);
                    terrain.push(C::from(elevation).unwrap());
                }
            }

            let runtime = now.elapsed();
            (terrain, runtime)
        };

        let distances_m: Vec<C> = linspace(C::zero(), distance_m, terrain_elev_m.len()).collect();

        let _earth_curve_runtime = {
            let now = std::time::Instant::now();
            if self.earth_curve {
                // https://www.trailnotes.org/SizeOfTheEarth/
                let earth_radius = C::from(crate::constants::MEAN_EARTH_RADIUS).unwrap();
                let start_elev_alt =
                    *terrain_elev_m.first().unwrap() + C::from(self.start_alt_m).unwrap();
                let start_radius_m = earth_radius + start_elev_alt;
                let end_elev_alt =
                    *terrain_elev_m.last().unwrap() + C::from(self.end_alt_m).unwrap();
                let elev_angle_rad = elevation_angle(start_elev_alt, distance_m, end_elev_alt);

                let (nb, nm) = if self.normalize {
                    let nb = -start_elev_alt;
                    let nm = (-end_elev_alt - nb) / distance_m;
                    (nb, nm)
                } else {
                    (C::zero(), C::zero())
                };

                for (&d_distance_m, elev_m) in distances_m.iter().zip(terrain_elev_m.iter_mut()) {
                    let radius_m = C::from(*elev_m).unwrap() + earth_radius;
                    // Approximate angle when radius is much larger than distance.
                    let chord_angle_rad = d_distance_m / radius_m;
                    let c_unk_unit = start_radius_m * (elev_angle_rad + C::FRAC_PI_2()).sin()
                        / (C::FRAC_PI_2() - elev_angle_rad - chord_angle_rad).sin();
                    let height_m = if self.normalize {
                        let los_m = -(nm * d_distance_m) - nb;
                        (radius_m - c_unk_unit) + los_m
                    } else {
                        radius_m - c_unk_unit
                    };
                    *elev_m = height_m;
                }
            }

            now.elapsed()
        };

        let los_elev_m = linspace(
            *terrain_elev_m.first().unwrap() + C::from(self.start_alt_m).unwrap(),
            *terrain_elev_m.last().unwrap() + C::from(self.end_alt_m).unwrap(),
            terrain_elev_m.len(),
        )
        .collect();

        debug!(
            "profile; len: {}, path_exec: {:?}, terrain_exec: {:?}",
            great_circle.len(),
            path_runtime,
            terrain_runtime
        );

        Ok(Profile {
            distances_m,
            great_circle,
            terrain_elev_m,
            los_elev_m,
        })
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::excessive_precision)]

    use super::{Coord, Profile, Tiles};
    use crate::tiles::TileMode;

    /// ```xml
    /// <?xml version="1.0" encoding="UTF-8"?>
    /// <kml xmlns="http://www.opengis.net/kml/2.2" xmlns:gx="http://www.google.com/kml/ext/2.2" xmlns:kml="http://www.opengis.net/kml/2.2" xmlns:atom="http://www.w3.org/2005/Atom">
    /// <Document>
    ///      <name>Mt Washington.kml</name>
    ///      <StyleMap id="m_ylw-pushpin">
    ///              <Pair>
    ///                      <key>normal</key>
    ///                      <styleUrl>#s_ylw-pushpin</styleUrl>
    ///              </Pair>
    ///              <Pair>
    ///                      <key>highlight</key>
    ///                      <styleUrl>#s_ylw-pushpin_hl</styleUrl>
    ///              </Pair>
    ///      </StyleMap>
    ///      <Style id="s_ylw-pushpin">
    ///              <IconStyle>
    ///                      <scale>1.1</scale>
    ///                      <Icon>
    ///                              <href>http://maps.google.com/mapfiles/kml/pushpin/ylw-pushpin.png</href>
    ///                      </Icon>
    ///                      <hotSpot x="20" y="2" xunits="pixels" yunits="pixels"/>
    ///              </IconStyle>
    ///      </Style>
    ///      <Style id="s_ylw-pushpin_hl">
    ///              <IconStyle>
    ///                      <scale>1.3</scale>
    ///                      <Icon>
    ///                              <href>http://maps.google.com/mapfiles/kml/pushpin/ylw-pushpin.png</href>
    ///                      </Icon>
    ///                      <hotSpot x="20" y="2" xunits="pixels" yunits="pixels"/>
    ///              </IconStyle>
    ///      </Style>
    ///      <Placemark>
    ///              <name>Mt Washington</name>
    ///              <styleUrl>#m_ylw-pushpin</styleUrl>
    ///              <LineString>
    ///                      <tessellate>1</tessellate>
    ///                      <coordinates>
    ///                              -71.30830716441369,44.28309806603165,0 -71.2972073283768,44.25628098424278,0
    ///                      </coordinates>
    ///              </LineString>
    ///      </Placemark>
    /// </Document>
    /// </kml>
    /// ```
    #[test]
    fn test_profile() {
        let start = Coord {
            x: -71.30830716441369,
            y: 44.28309806603165,
        };
        let end = Coord {
            x: -71.2972073283768,
            y: 44.25628098424278,
        };

        let tile_source = Tiles::new(crate::three_arcsecond_dir(), TileMode::MemMap).unwrap();

        let _90m = 90.0;
        let profile = Profile::builder()
            .start(start)
            .max_step(_90m)
            .end(end)
            .build(&tile_source)
            .unwrap();
        assert_eq!(36, profile.great_circle.len());
    }
}
