use axum::{extract::{Path, State}, http::StatusCode, response::{IntoResponse, Response}};
use std::{sync::{Arc, RwLock}, io::Cursor};
use crate::world_grid::WorldGrid;
use image::{ImageFormat, RgbaImage, DynamicImage};
use bytes::Bytes;

pub async fn handler(
    Path((z, x, y)): Path<(u8, u32, u32)>,
    State(world): State<Arc<RwLock<WorldGrid>>>,
) -> Response {
    let world = match world.read() {
        Ok(w) => w,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    if z < 13 {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    if let Some(tile) = world.get_tile_img(x, y, z) {
        let mut buf = Cursor::new(Vec::new());
        if DynamicImage::ImageRgba8(tile)
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
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}
