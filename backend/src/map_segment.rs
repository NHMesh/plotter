use gdal::{Dataset, raster::Buffer, errors::GdalError };
use image::{Rgba, RgbaImage, imageops, imageops::FilterType};
use palette::Srgba;
use std::{f32, error::Error};
use geo::{algorithm::bounding_rect::BoundingRect, prelude::Contains, Polygon, Coord};

pub struct MapSegment {
    pub origin_x: f64,
    pub origin_y: f64,
    pub pixel_width: f64,
    pub pixel_height: f64,
    pub width: usize,
    pub height: usize,
    pub height_map: Buffer<f32>,
    pub img: RgbaImage,
}

impl MapSegment {
    pub fn load_from_dataset(path :&str) -> Result<Self, GdalError> {
        let dataset = Dataset::open(path)?;

        let transform = dataset.geo_transform()?;
        let band = dataset.rasterband(1)?;

        let width = band.x_size();
        let height = band.y_size();

        let height_map = band.read_as::<f32>((0, 0), (width, height), (width, height), None)?;

        let img = RgbaImage::new(width as u32, height as u32);
        let origin_x = transform[0];
        let pixel_width = transform[1];
        let origin_y = transform[3];
        let pixel_height = transform[5];

        println!("This {} is in '{}' and has {} bands.", dataset.driver().long_name(), dataset.spatial_ref()?.name().unwrap(), dataset.raster_count());
        println!("origin_x({}) origin_y({}) pixel_width({}) pixel_height({})", origin_x, origin_y, pixel_width, pixel_height);

        Ok(Self {
            origin_x,
            pixel_width,
            origin_y,
            pixel_height,
            width,
            height,
            height_map,
            img,
        })
    }

    pub fn scan_polygon(&self, polygon: &[(f64, f64)]) -> Vec<(usize, usize, f32)> {
        let pixel_coords: Vec<Coord<usize>> = polygon
            .iter()
            .map(|&(lat, lon)| {
                let (x, y) = self.latlon_to_pixel(lat, lon);
                Coord { x, y }
            })
        .collect();

        let exterior = pixel_coords.iter().map(|c| (c.x as f64, c.y as f64)).collect::<Vec<_>>();
        let geo_polygon = Polygon::new(exterior.into(), vec![]);

        let mut result = Vec::new();

        if let Some(bbox) = geo_polygon.bounding_rect() {
            let min_x = bbox.min().x.floor().max(0.0) as usize;
            let max_x = bbox.max().x.ceil().min(self.width as f64) as usize;
            let min_y = bbox.min().y.floor().max(0.0) as usize;
            let max_y = bbox.max().y.ceil().min(self.height as f64) as usize;

            for y in min_y..max_y {
                for x in min_x..max_x {
                    let pt = geo::Point::new(x as f64, y as f64);
                    if geo_polygon.contains(&pt) {
                        if let Some(elev) = self.get_elevation(x, y) {
                            result.push((x, y, elev));
                        }
                    }
                }
            }
        }

        result
    }


    pub fn gen_map(&mut self)  -> Result<(), Box<dyn Error>> {
        // Find min and max elevation and their positions
        let mut min = f32::INFINITY;
        let mut max = f32::NEG_INFINITY;

        for (_i, &val) in self.height_map.data.iter().enumerate() {
            if val < min {
                min = val;
            }
            if val > max {
                max = val
            }
        }

        let scale = 1.0 / (max - min);
        println!("min({}) max({}) scale({})", min, max, scale);

        // Render Image
        for y in 0..self.height {
            for x in 0..self.width {
                let i = y * self.width + x;
                if i % 1000 == 0 {
                    println!("{}/{}", i/1000000, (self.height as f64 * self.width as f64)/1000000.0);
                }
                let mut rgb: Srgba<f32> = Srgba::new(0.0, 0.0, 0.0, 255.0);
                let elevation = self.height_map.data[i];

                if !within_banded_range(elevation, 10, 1.0) {
                    //let norm = (elevation - min) * -scale;
                    //rgb = Srgb::from(Hsl::new(norm * 255.0, 1.0, 0.5));
                    rgb = Srgba::new(255.0,255.0,255.0, 0.0);
                }

                self.img.put_pixel(
                    x as u32,
                    y as u32,
                    Rgba([
                        (rgb.red * 255.0) as u8,
                        (rgb.green * 255.0) as u8,
                        (rgb.blue * 255.0) as u8,
                        (rgb.alpha) as u8,
                    ]),
                );
            }
        }

        Ok(())
    }

    pub fn draw_pixel(&mut self, x: u32, y: u32) {
        self.img.put_pixel(
            x,
            y,
            Rgba([ 252, 3, 248, 255 ]),
        );
    }

    pub fn get_tile_los<F>(&self, x: u32, y: u32, zoom: u8, observer_lat: f64, observer_lon: f64, mut tick: F)
    where
        F: FnMut(usize, usize, bool)
    {
        println!("\nGenerating Tile Data for x({}), y({}), zoom({})", x, y, zoom);
        const TILE_SIZE: usize = 256;

        let ((lat_north, lon_west),
            (lat_south, lon_east)) = tile_bbox_latlon(x, y, zoom);

        let (px_min, py_min) = self.latlon_to_pixel(lat_north, lon_west);
        let (px_max, py_max) = self.latlon_to_pixel(lat_south, lon_east);

        let step_size_x: f32 = ( px_max - px_min ) as f32 / TILE_SIZE as f32;
        let step_size_y: f32 = ( py_max - py_min ) as f32 / TILE_SIZE as f32;

        println!("step_size_x({}), step_size_y({})", step_size_x, step_size_y);

        let px_min_f32 = px_min as f32;
        let py_min_f32 = py_min as f32;
        //let px_max_f32 = px_max as f32;
        //let py_max_f32 = py_max as f32;

        for x in 0..TILE_SIZE {
            for y in 0..TILE_SIZE {
                let source_x = px_min_f32 + ( x as f32 * step_size_x);
                let source_y = py_min_f32 + (y as f32 * step_size_y);

                let (lat, lon) = self.pixel_to_latlon(source_x.round() as usize, source_y.round() as usize);

                let has_los = self.has_line_of_sight(
                    observer_lat,
                    observer_lon,
                    lat,
                    lon,
                    1.0,
                    1.0
                );

                tick(x, y, has_los);
            }
        }
    }

    pub fn get_tile_data<F>(&self, x: u32, y: u32, zoom: u8, mut tick: F)
    where
        F: FnMut(usize, usize, f32)
    {
        println!("\nGenerating Tile Data for x({}), y({}), zoom({})", x, y, zoom);
        const TILE_SIZE: usize = 256;

        let ((lat_north, lon_west),
            (lat_south, lon_east)) = tile_bbox_latlon(x, y, zoom);

        let (px_min, py_min) = self.latlon_to_pixel(lat_north, lon_west);
        let (px_max, py_max) = self.latlon_to_pixel(lat_south, lon_east);

        let step_size_x: f32 = ( px_max - px_min ) as f32 / TILE_SIZE as f32;
        let step_size_y: f32 = ( py_max - py_min ) as f32 / TILE_SIZE as f32;

        println!("step_size_x({}), step_size_y({})", step_size_x, step_size_y);

        let px_min_f32 = px_min as f32;
        let py_min_f32 = py_min as f32;
        //let px_max_f32 = px_max as f32;
        //let py_max_f32 = py_max as f32;

        for x in 0..TILE_SIZE {
            for y in 0..TILE_SIZE {
                let source_x = px_min_f32 + ( x as f32 * step_size_x);
                let source_y = py_min_f32 + (y as f32 * step_size_y);

                tick(x, y, self.get_elevation(source_x.round() as usize, source_y.round() as usize).unwrap_or(0.0));
            }
        }
    }

    pub fn get_tile_img(&self, x: u32, y: u32, zoom: u8) -> RgbaImage {
        println!("\nRendering Tile x({}), y({}), zoom({})", x, y, zoom);
        let tile_size = 256;
        let ((lat_north, lon_west), (lat_south, lon_east)) = tile_bbox_latlon(x, y, zoom);

        println!("lat_north({}) lon_east({}) lat_south({}) lon_west({})", lat_north, lon_east, lat_south, lon_west);
        println!("origin_x({}) origin_y({})", self.origin_x, self.origin_y);

        // Convert lat/lon to pixel coordinates in raster
        let px_min = ((lon_west - self.origin_x) / self.pixel_width).floor() as i32;
        let px_max = ((lon_east - self.origin_x) / self.pixel_width).ceil() as i32;
        let py_min = ((lat_north - self.origin_y) / self.pixel_height).floor() as i32;
        let py_max = ((lat_south - self.origin_y) / self.pixel_height).ceil() as i32;

        println!("px_min({}) px_max({}) py_min({}) py_max({})", px_min, px_max, py_min, py_max);

        // Make a blank white tile
        //let mut tile = RgbaImage::from_pixel(tile_size, tile_size, Rgba([255, 255, 255, 127]));

        let cropped = imageops::crop_imm(&self.img, px_min as u32, py_min as u32, ( px_max - px_min ) as u32, ( py_max - py_min ) as u32).to_image();

        image::imageops::resize(&cropped, tile_size, tile_size, FilterType::Lanczos3)
    }

    pub fn has_line_of_sight(
        &self,
        from_lat: f64, // Latitude of the observer
        from_lon: f64, // Longitude of the observer
        to_lat: f64, // Latitude of the target
        to_lon: f64, // Longitude of the target
        observer_height: f32, // Height of the observer above ground
        target_height: f32, // Height of the target above ground
    ) -> bool {
        let (x0, y0) = self.latlon_to_pixel(from_lat, from_lon); // Convert observer's lat/lon to pixel coordinates
        let (x1, y1) = self.latlon_to_pixel(to_lat, to_lon); // Convert target's lat/lon to pixel coordinates

        let z0 = self.get_elevation(x0, y0).unwrap_or(0.0) + observer_height; // Elevation at observer's position
        let z1 = self.get_elevation(x1, y1).unwrap_or(0.0) + target_height; // Elevation at target's position

        let dx = x1 as isize - x0 as isize; // Difference in x-coordinates
        let dy = y1 as isize - y0 as isize; // Difference in y-coordinates
        let steps = dx.abs().max(dy.abs()); // Number of steps for interpolation

        for i in 1..steps {
            let t = i as f32 / steps as f32; // Interpolation factor
            let xi = x0 as f32 + t * dx as f32; // Interpolated x-coordinate
            let yi = y0 as f32 + t * dy as f32; // Interpolated y-coordinate
            let zi = z0 + t * (z1 - z0); // Interpolated elevation

            let elev = self.get_elevation(xi as usize, yi as usize).unwrap_or(0.0); // Elevation at interpolated position
            if elev > zi {
                return false; // Line of sight is blocked
            }
        }

        true // Line of sight is clear
    }

    pub fn generate_los_map(&mut self, _observer_lat: f64, _observer_lon: f64, _radius: usize, _observer_height: f32) {
        // Stub: los_map logic moved to WorldGrid
    }

    pub fn generate_los_map_for_world<'a, F>(&mut self, _segment_lookup: F, _observer_lat: f64, _observer_lon: f64, _radius: usize, _observer_height: f32)
    where
        F: Fn(f64, f64) -> Option<&'a MapSegment>,
    {
        // Stub: los_map logic moved to WorldGrid
    }

    pub fn latlon_to_pixel(&self, lat: f64, lon: f64) -> (usize, usize) {
        let px = ((lon - self.origin_x) / self.pixel_width) as usize;
        let py = ((lat - self.origin_y) / self.pixel_height) as usize;

        (px, py)
    }

    pub fn pixel_to_latlon(&self, px: usize, py: usize) -> (f64, f64) {
        let lon = self.origin_x + px as f64 * self.pixel_width;
        let lat = self.origin_y + py as f64 * self.pixel_height;
        (lat, lon)
    }

    pub fn get_elevation(&self, x: usize, y: usize) -> Option<f32> {
        if x < self.width && y < self.height {
            let i = y * self.width + x;
            Some(self.height_map.data[i])
        } else {
            None
        }
    }

}

fn within_banded_range(n: f32, multiple: i32, buffer: f64) -> bool {
    let remainder = ( n as f64 ) % multiple as f64;
    remainder <= buffer || remainder >= ( multiple as f64 - buffer )
}

fn tile_to_latlon(x: u32, y: u32, zoom: u8) -> (f64, f64) {
    let n = 2f64.powi(zoom as i32);
    let lon_deg = x as f64 / n * 360.0 - 180.0;
    let lat_rad = ((1.0 - 2.0 * y as f64 / n) * std::f64::consts::PI).sinh().atan();
    let lat_deg = lat_rad.to_degrees();
    (lat_deg, lon_deg)
}

pub fn tile_bbox_latlon(x: u32, y: u32, zoom: u8) -> ((f64, f64), (f64, f64)) {
    // Top-left corner
    let (lat1, lon1) = tile_to_latlon(x, y, zoom);
    // Bottom-right corner (x+1, y+1)
    let (lat2, lon2) = tile_to_latlon(x + 1, y + 1, zoom);

    // Return ((north, west), (south, east)) as ((lat_max, lon_min), (lat_min, lon_max))
    (
        (lat1.max(lat2), lon1.min(lon2)), // top-left (NW)
        (lat1.min(lat2), lon1.max(lon2)), // bottom-right (SE)
    )
}
