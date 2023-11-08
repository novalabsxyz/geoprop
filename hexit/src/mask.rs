use anyhow::Result;
use geo::GeometryCollection;
use geojson::{quick_collection, GeoJson};
use std::{fs::File, path::Path};

pub fn open(maybe_path: Option<&Path>) -> Result<Option<GeometryCollection>> {
    match maybe_path {
        None => Ok(None),
        Some(path) => {
            let mask_file = File::open(path)?;
            let mask_json = GeoJson::from_reader(mask_file)?;
            let mask: GeometryCollection<f64> = quick_collection(&mask_json)?;
            Ok(Some(mask))
        }
    }
}
