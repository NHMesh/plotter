use axum::{extract::{Path, State}, http::StatusCode, response::{IntoResponse, Response}};
use std::{sync::{Arc, RwLock}, io::Cursor};
use crate::map_segment::MapSegment;
use image::{ImageFormat, RgbaImage, DynamicImage};
use bytes::Bytes;

pub async fn handler(
    Path((z, x, y)): Path<(u8, u32, u32)>,
    State(ms): State<Arc<RwLock<MapSegment>>>,
) -> Response {
    let ms = match ms.read() {
        Ok(m) => m,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };

    let tile: RgbaImage = ms.get_tile(x, y, z);
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
}
