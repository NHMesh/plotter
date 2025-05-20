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

    ms.get_tile_data(x, y, z, |x,y,elevation| {
        let draw = match z {
            13..=255 => within_banded_range(elevation, 20, 1.0),
            12..13 => within_banded_range(elevation, 30, 2.0),
            11..12 => within_banded_range(elevation, 60, 2.0),
            0..11 => false
        };

        if draw {
            img.put_pixel(x as u32, y as u32, Rgba([0, 0, 0, 255]));
        }
    });

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

fn within_banded_range(n: f32, multiple: i32, buffer: f64) -> bool {
    let remainder = ( n as f64 ) % multiple as f64;
    remainder <= buffer || remainder >= ( multiple as f64 - buffer )
}
