use crate::map_segment::{MapSegment, tile_bbox_latlon};
use image::RgbaImage;

pub struct WorldGrid {
    pub segments: Vec<MapSegment>,
}

impl WorldGrid {
    pub fn new() -> Self {
        Self { segments: Vec::new() }
    }

    pub fn add_segment(&mut self, segment: MapSegment) {
        self.segments.push(segment);
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
