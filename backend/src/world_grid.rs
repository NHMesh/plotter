use crate::map_segment::{MapSegment, tile_bbox_latlon};
use crate::los_map::LosMap;
use image::RgbaImage;

pub struct WorldGrid {
    pub segments: Vec<MapSegment>,
    pub los_maps: Vec<LosMap>,
}

impl WorldGrid {
    pub fn new() -> Self {
        Self { segments: Vec::new(), los_maps: Vec::new() }
    }

    pub fn add_segment(&mut self, segment: MapSegment) {
        self.segments.push(segment);
    }

    pub fn add_los_map(&mut self, los_map: LosMap) {
        self.los_maps.push(los_map);
    }

    pub fn get_los_map(&self, idx: usize) -> Option<&LosMap> {
        self.los_maps.get(idx)
    }

    /// Generate and add a new LOS map to the global los_maps vector.
    /// Returns the index of the new LOS map.
    pub fn generate_and_add_los_map(
        &mut self,
        observer_lat: f64,
        observer_lon: f64,
        radius: usize,
        observer_height: f32,
    ) -> usize
    {
        let mut los_map = LosMap::new(radius, radius);
        los_map.generate(self, observer_lat, observer_lon, radius, observer_height);
        self.los_maps.push(los_map);
        self.los_maps.len() - 1
    }

    /// Load a MapSegment from a dataset path and add it to the grid. Returns Result<(), GdalError>.
    pub fn load_segment(&mut self, path: &str) -> Result<(), gdal::errors::GdalError> {
        let segment = MapSegment::load_from_dataset(path)?;
        self.add_segment(segment);
        Ok(())
    }

    /// Find the segment containing the given lat/lon, or None if not found
    pub fn find_segment(&self, lat: f64, lon: f64) -> Option<&MapSegment> {
        self.segments.iter().find(|seg| {
            let (lat_north, lon_west) = seg.pixel_to_latlon(0, 0);
            let (lat_south, lon_east) = seg.pixel_to_latlon(seg.width - 1, seg.height - 1);
            lat <= lat_north && lat >= lat_south && lon >= lon_west && lon <= lon_east
        })
    }

    /// Generate a tile for the given z/x/y by searching all segments
    pub fn get_tile_img(&self, x: u32, y: u32, zoom: u8) -> Option<RgbaImage> {
        for seg in &self.segments {
            let ((lat_north, lon_west), (lat_south, lon_east)) = tile_bbox_latlon(x, y, zoom);
            let (seg_lat_north, seg_lon_west) = seg.pixel_to_latlon(0, 0);
            let (seg_lat_south, seg_lon_east) = seg.pixel_to_latlon(seg.width - 1, seg.height - 1);
            let overlap = lat_north >= seg_lat_south && lat_south <= seg_lat_north &&
                          lon_east >= seg_lon_west && lon_west <= seg_lon_east;
            if overlap {
                return Some(seg.get_tile_img(x, y, zoom));
            }
        }
        None
    }
}
