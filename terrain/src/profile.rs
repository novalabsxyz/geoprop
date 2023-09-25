use crate::{haversine::HaversineIter, math::Atan2, TerrainError, Tiles};
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

impl<C: CoordFloat + FromPrimitive + Atan2> Profile<C> {
    pub fn new(
        start @ Coord {
            x: start_x,
            y: start_y,
        }: Coord<C>,
        step_size_m: C,
        end: Coord<C>,
        tiles: &Tiles,
    ) -> Result<Self, TerrainError>
    where
        C: FromPrimitive,
        f64: From<C>,
    {
        let start_point = Point::from(start);
        let end_point = Point::from(end);

        let distance = start_point.haversine_distance(&end_point);

        let (path, path_runtime) = {
            let now = std::time::Instant::now();
            let path: Vec<Point<C>> =
                HaversineIter::new(Point::from(start), step_size_m, Point::from(end)).collect();
            let runtime = now.elapsed();
            (path, runtime)
        };

        let (terrain, terrain_runtime) = {
            let mut terrain = Vec::with_capacity(path.len());
            let now = std::time::Instant::now();
            let mut tile = tiles.get(Coord {
                x: start_x.into(),
                y: start_y.into(),
            })?;
            for point in &path {
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
            let runtime = now.elapsed();
            (terrain, runtime)
        };

        debug!(
            "profile; len: {}, path_exec: {:?}, terrain_exec: {:?}",
            path.len(),
            path_runtime,
            terrain_runtime
        );

        Ok(Self {
            distance,
            great_circle: path,
            terrain,
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
        let profile = Profile::new(start, _90m, end, &tile_source).unwrap();
        println!("{:#?}", profile);
        assert_eq!(36, profile.great_circle.len());
    }
}
