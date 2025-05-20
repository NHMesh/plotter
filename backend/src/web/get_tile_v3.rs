use axum::{extract::{Path, State}, http::StatusCode, response::{IntoResponse, Response}};
use std::{sync::{Arc, RwLock}, io::Cursor};
use crate::map_segment::MapSegment;
use image::{ImageFormat, RgbaImage, ImageBuffer, Rgba, DynamicImage};
use bytes::Bytes;

pub async fn handler(
    Path((z, x, y)): Path<(u8, u32, u32)>,
    State(ms): State<Arc<RwLock<MapSegment>>>,
) -> Response {
    let ms = match ms.read() {
        Ok(m) => m,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let mut img: RgbaImage = ImageBuffer::from_pixel(256, 256, Rgba([255, 255, 255, 0]));

    // Only proceed if los_map is present
    if let Some(los_map) = &ms.los_map {
        // Calculate tile bounds in raster pixel space
        let ((lat_north, lon_west), (lat_south, lon_east)) = crate::map_segment::tile_bbox_latlon(x, y, z);
        let (px_min, py_min) = ms.latlon_to_pixel(lat_north, lon_west);
        let (px_max, py_max) = ms.latlon_to_pixel(lat_south, lon_east);
        let step_size_x: f32 = (px_max - px_min) as f32 / 256.0;
        let step_size_y: f32 = (py_max - py_min) as f32 / 256.0;
        let px_min_f32 = px_min as f32;
        let py_min_f32 = py_min as f32;

        for x in 0..256 {
            for y in 0..256 {
                let source_x = px_min_f32 + (x as f32 * step_size_x);
                let source_y = py_min_f32 + (y as f32 * step_size_y);
                let px = source_x.round() as usize;
                let py = source_y.round() as usize;
                if px < ms.width && py < ms.height {
                    let idx = py * ms.width + px;
                    if idx < los_map.len() && los_map[idx] {
                        img.put_pixel(x as u32, y as u32, Rgba([190, 0, 150, 255]));
                    }
                }
            }
        }
    }

    let mut buf = Cursor::new(Vec::new());
    if DynamicImage::ImageRgba8(img)
        .write_to(&mut buf, ImageFormat::Png)
        .is_err()
    {
        println!("Failed to encode tile PNG");
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let bytes = Bytes::from(buf.into_inner());
    (
        [
            ("Content-Type", "image/png"),
            ("Content-Length", &bytes.len().to_string()),
        ],
        bytes
    ).into_response()
}
