use std::f64::consts::TAU;
use crate::map_segment::MapSegment;
use crate::world_grid::WorldGrid;

pub struct LosMap {
    pub data: Vec<bool>,
    pub width: usize,
    pub height: usize,
}

impl LosMap {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            data: vec![false; width * height],
            width,
            height,
        }
    }

    /// Efficiently generate a line-of-sight map from the observer (in lat/lon) out to a given radius (in pixels)
    /// Now accounts for Earth's curvature.
    pub fn generate(
        &mut self,
        world: &WorldGrid,
        observer_lat: f64,
        observer_lon: f64,
        radius: usize,
        observer_height: f32,
    ) {
        // Find the segment containing the observer
        let segment = world.find_segment(observer_lat, observer_lon)
            .expect("No segment contains the observer location");
        let (obs_px, obs_py) = segment.latlon_to_pixel(observer_lat, observer_lon);
        let obs_elev = segment.get_elevation(obs_px, obs_py).unwrap_or(0.0) + observer_height;

        // Constants for Earth's curvature
        const EARTH_RADIUS_M: f64 = 6_371_000.0; // meters
        let pixel_size_m = segment.pixel_width.hypot(segment.pixel_height).abs(); // crude average pixel size in meters

        let num_rays = 360.max(radius * 8); // More rays for larger radius
        for angle_step in 0..num_rays {
            let theta = (angle_step as f64) * TAU / (num_rays as f64);
            let dx = theta.cos();
            let dy = theta.sin();
            let mut max_angle = std::f32::NEG_INFINITY;
            for r in 1..=radius {
                // Compute lat/lon for this step
                let xi = obs_px as f64 + dx * r as f64;
                let yi = obs_py as f64 + dy * r as f64;
                let (lat, lon) = segment.pixel_to_latlon(xi.round() as usize, yi.round() as usize);
                // Find the segment for this lat/lon
                if let Some(seg) = world.find_segment(lat, lon) {
                    let (x, y) = seg.latlon_to_pixel(lat, lon);
                    if x >= seg.width || y >= seg.height {
                        break;
                    }
                    let idx = y * self.width + x;
                    let elev = seg.get_elevation(x, y).unwrap_or(0.0);
                    let dist_m = (r as f64) * pixel_size_m;
                    let curvature_drop = dist_m * dist_m / (2.0 * EARTH_RADIUS_M);
                    let adj_elev = elev - curvature_drop as f32;
                    let dist = dist_m as f32;
                    let angle = if dist > 0.0 { (adj_elev - obs_elev) / dist } else { std::f32::NEG_INFINITY };
                    if angle > max_angle {
                        self.data[idx] = true;
                        max_angle = angle;
                    }
                } else {
                    break;
                }
            }
        }
    }

}
