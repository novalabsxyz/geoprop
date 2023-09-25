use crate::{
    math::{Atan2, HaversineIter},
    TerrainError, Tiles,
};
use geo::{
    algorithm::HaversineDistance,
    geometry::{Coord, Point},
    CoordFloat,
};
use log::debug;
use num_traits::cast::FromPrimitive;

#[derive(Debug, Clone, PartialEq)]
pub struct Profile<C: CoordFloat = f32> {
    /// Total distance from `start` to `end` in meters.
    pub distance: C,

    /// Location of step along the great circle route from `start` to
    /// `end`.
    pub great_circle: Vec<Point<C>>,

    /// Elevation at each step along the great circle route from
    /// `start` to `end`.
    pub terrain: Vec<i16>,
}

impl<C> Profile<C>
where
    C: CoordFloat,
{
    pub fn builder() -> ProfileBuilder<C> {
        ProfileBuilder {
            start: None,
            end: None,
            start_alt_m: None,
            step_size_m: None,
            end_alt_m: None,
            earth_curve: false,
            normalize: false,
        }
    }
}

pub struct ProfileBuilder<C: CoordFloat = f32> {
    start: Option<Coord<C>>,

    end: Option<Coord<C>>,

    /// Starting altitude above ground (meters).
    start_alt_m: Option<i16>,

    /// Maximum distance between points.
    step_size_m: Option<C>,

    /// Starting altitude above ground (meters).
    end_alt_m: Option<i16>,

    /// Add earth curvature.
    earth_curve: bool,

    /// Place virtual earth curve as the highest and center point of
    /// the output.
    normalize: bool,
}

impl<C> ProfileBuilder<C>
where
    C: Atan2 + CoordFloat + FromPrimitive,
    f64: From<C>,
{
    pub fn start(mut self, coord: Coord<C>) -> Self {
        self.start = Some(coord);
        self
    }

    pub fn start_alt(mut self, meters: i16) -> Self {
        self.start_alt_m = Some(meters);
        self
    }

    pub fn step_size(mut self, meters: C) -> Self {
        self.step_size_m = Some(meters);
        self
    }

    pub fn end(mut self, coord: Coord<C>) -> Self {
        self.end = Some(coord);
        self
    }

    pub fn end_alt(mut self, meters: i16) -> Self {
        self.end_alt_m = Some(meters);
        self
    }

    pub fn earth_curve(mut self, add_curve: bool) -> Self {
        self.earth_curve = add_curve;
        self
    }

    pub fn normalize(mut self, normalize: bool) -> Self {
        self.normalize = normalize;
        self
    }

    pub fn build(&self, tiles: &Tiles) -> Result<Profile<C>, TerrainError>
    where
        C: Atan2 + FromPrimitive,
    {
        if let (Some(start), Some(step_size_m), Some(end)) =
            (self.start, self.step_size_m, self.end)
        {
            let start_point = Point::from(start);
            let end_point = Point::from(end);

            let distance = start_point.haversine_distance(&end_point);

            let (great_circle, path_runtime) = {
                let now = std::time::Instant::now();
                let great_circle: Vec<Point<C>> =
                    HaversineIter::new(Point::from(start), step_size_m, Point::from(end)).collect();
                let runtime = now.elapsed();
                (great_circle, runtime)
            };

            let (terrain, terrain_runtime) = {
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
                        terrain.push(elevation);
                    } else {
                        tile = tiles.get(coord)?;
                        let elevation = tile.get_unchecked(coord);
                        terrain.push(elevation);
                    }
                }

                // Add optional height obove ground for start point.
                if let (Some(elev), Some(start_alt_m)) = (terrain.first_mut(), self.start_alt_m) {
                    *elev += start_alt_m;
                };

                // Add optional height obove ground for end point.
                if let (Some(elev), Some(end_alt_m)) = (terrain.last_mut(), self.end_alt_m) {
                    *elev += end_alt_m;
                };

                let runtime = now.elapsed();
                (terrain, runtime)
            };

            debug!(
                "profile; len: {}, path_exec: {:?}, terrain_exec: {:?}",
                great_circle.len(),
                path_runtime,
                terrain_runtime
            );

            Ok(Profile {
                distance,
                great_circle,
                terrain,
            })
        } else {
            Err(TerrainError::Builder)
        }
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
            .step_size(_90m)
            .end(end)
            .build(&tile_source)
            .unwrap();
        println!("{:#?}", profile);
        assert_eq!(36, profile.great_circle.len());
    }
}
