use std::f64::consts::TAU;
use palette::encoding::pixel;

use crate::map_segment::MapSegment;
use crate::world_grid::WorldGrid;

pub struct LosMap {
    pub data: Vec<bool>,
    pub width: usize,
    pub height: usize,
    pub origin_x: f64,      // longitude of the NW (top-left) corner
    pub origin_y: f64,      // latitude of the NW (top-left) corner
    pub pixel_width: f64,   // width of a pixel in degrees (longitude)
    pub pixel_height: f64,  // height of a pixel in degrees (latitude)
}

impl LosMap {
    /// Create and generate a LOS map for the given observer and radius, using the world grid for georeferencing.
    pub fn new(
        world: &WorldGrid,
        observer_lat: f64,
        observer_lon: f64,
        radius: usize,
        observer_height: f32,
    ) -> Self {
        // Find the segment containing the observer
        let segment = world.find_segment(observer_lat, observer_lon)
            .expect("No segment contains the observer location");
        let (obs_px, obs_py) = segment.latlon_to_pixel(observer_lat, observer_lon);
        let obs_elev = segment.get_elevation(obs_px, obs_py).unwrap_or(0.0) + observer_height;

        // Set georeferencing for this LOS map
        let pixel_width = segment.pixel_width;
        let pixel_height = segment.pixel_height;
        let origin_x = segment.origin_x + (obs_px as f64 - radius as f64) * pixel_width;
        let origin_y = segment.origin_y + (obs_py as f64 - radius as f64) * pixel_height;
        let width = 2 * radius + 1;
        let height = 2 * radius + 1;
        let mut data = vec![false; width * height];

        // Constants for Earth's curvature
        const EARTH_RADIUS_M: f64 = 6_371_000.0; // meters
        let pixel_size_m = segment.pixel_width.hypot(segment.pixel_height).abs(); // crude average pixel size in meters
        print!("{}", pixel_size_m);

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
                    // Map segment pixel to los_map pixel
                    let los_x = (x as isize + (obs_px as isize - radius as isize) - obs_px as isize + radius as isize) as usize;
                    let los_y = (y as isize + (obs_py as isize - radius as isize) - obs_py as isize + radius as isize) as usize;
                    if los_x < width && los_y < height {
                        let idx = los_y * width + los_x;
                        let elev = seg.get_elevation(x, y).unwrap_or(0.0);
                        let dist_m = (r as f64) * pixel_size_m;
                        let curvature_drop = dist_m * dist_m / (2.0 * EARTH_RADIUS_M);
                        let adj_elev = elev - curvature_drop as f32;
                        let dist = dist_m as f32;
                        let angle = if dist > 0.0 { (adj_elev - obs_elev) / dist } else { std::f32::NEG_INFINITY };
                        if angle > max_angle {
                            data[idx] = true;
                            max_angle = angle;
                        }
                    }
                } else {
                    break;
                }
            }
        }

        Self {
            data,
            width,
            height,
            origin_x,
            origin_y,
            pixel_width,
            pixel_height,
        }
    }

    /// Convert lat/lon to pixel coordinates in the LOS map
    pub fn latlon_to_pixel(&self, lat: f64, lon: f64) -> (usize, usize) {
        let px = ((lon - self.origin_x) / self.pixel_width).round() as usize;
        let py = ((lat - self.origin_y) / self.pixel_height).round() as usize;
        (px, py)
    }

    /// Convert pixel coordinates to lat/lon in the LOS map
    pub fn pixel_to_latlon(&self, px: usize, py: usize) -> (f64, f64) {
        let lon = self.origin_x + px as f64 * self.pixel_width;
        let lat = self.origin_y + py as f64 * self.pixel_height;
        (lat, lon)
    }
}
