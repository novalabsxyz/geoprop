use crate::{TerrainError, TileSource};
use geo::{
    algorithm::geodesic_intermediate::GeodesicIntermediate,
    geometry::{Coord, Point},
};

#[derive(Debug, Clone, PartialEq)]
pub struct Profile {
    pub points: Vec<Point<f64>>,
    pub terrain: Vec<i16>,
}

impl Profile {
    pub fn new(
        start: Coord<f64>,
        end: Coord<f64>,
        tiles: &TileSource,
    ) -> Result<Self, TerrainError> {
        let points = GeodesicIntermediate::geodesic_intermediate_fill(
            &Point::from(start),
            &Point::from(end),
            30.0,
            true,
        );

        let mut terrain = Vec::with_capacity(points.len());
        for point in points.iter() {
            let maybe_tile = tiles.get(point.0)?;
            let elevation = maybe_tile.and_then(|tile| tile.get(point.0)).unwrap_or(0);
            terrain.push(elevation)
        }

        Ok(Self { points, terrain })
    }
}

#[cfg(test)]
mod tests {
    use super::{Coord, Point, Profile, TileSource};
    use plotters::prelude::*;
    use std::path::Path;

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

        let tile_source = TileSource::new(crate::three_arcsecond_dir()).unwrap();
        let profile = Profile::new(start, end, &tile_source).unwrap();
        profile.plot("/tmp/path.png");
    }

    impl Profile {
        pub fn plot<P: AsRef<Path>>(&self, path: P) {
            let root = BitMapBackend::new(&path, (1024, 500)).into_drawing_area();
            root.fill(&WHITE).unwrap();
            let Point(Coord {
                x: start_x,
                y: start_y,
            }) = self.points.first().unwrap();
            let Point(Coord { x: end_x, y: end_y }) = self.points.first().unwrap();
            let caption = format!("{:6},{:6} to {:6},{:6}", start_y, start_x, end_x, end_y);
            let mut chart = ChartBuilder::on(&root)
                .caption(caption, ("sans-serif", 12).into_font())
                .margin(5)
                .x_label_area_size(30)
                .y_label_area_size(30)
                .build_cartesian_2d(
                    0f32..(self.terrain.len() as f32 * 30.0f32),
                    1300f32..2000f32,
                )
                .unwrap();

            chart.configure_mesh().draw().unwrap();

            chart
                .draw_series(LineSeries::new(
                    self.terrain
                        .iter()
                        .enumerate()
                        .map(|(idx, elev)| (idx as f32 * 30.0, *elev as f32)),
                    &RED,
                ))
                .unwrap()
                .label("Elevation")
                .legend(|(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], RED));

            chart
                .configure_series_labels()
                .background_style(WHITE.mix(0.8))
                .border_style(BLACK)
                .draw()
                .unwrap();

            root.present().unwrap();
        }
    }
}
