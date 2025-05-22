// GET /los_map_img/{idx}
use axum::{extract::{Path, State}, http::StatusCode, response::{IntoResponse, Response}};
use std::sync::{Arc, RwLock};
use crate::world_grid::WorldGrid;
use image::{ImageFormat, Rgba, RgbaImage, DynamicImage};
use bytes::Bytes;

pub async fn handler(
    Path(idx): Path<usize>,
    State(world): State<Arc<RwLock<WorldGrid>>>,
) -> Response {
    let world = match world.read() {
        Ok(w) => w,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let los_map = match world.los_maps.get(idx) {
        Some(lm) => lm,
        None => return StatusCode::NOT_FOUND.into_response(),
    };
    let mut img: RgbaImage = RgbaImage::from_pixel(los_map.width as u32, los_map.height as u32, Rgba([255, 255, 255, 0]));
    for y in 0..los_map.height {
        for x in 0..los_map.width {
            let idx = y * los_map.width + x;
            if idx < los_map.data.len() && los_map.data[idx] {
                img.put_pixel(x as u32, y as u32, Rgba([190, 0, 150, 255]));
            }
        }
    }
    let mut buf = std::io::Cursor::new(Vec::new());
    if DynamicImage::ImageRgba8(img)
        .write_to(&mut buf, ImageFormat::Png)
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    let bytes = Bytes::from(buf.into_inner());
    ([
        ("Content-Type", "image/png"),
        ("Content-Length", &bytes.len().to_string()),
    ], bytes).into_response()
}
