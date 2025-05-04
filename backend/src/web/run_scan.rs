// src/web/scan.rs

use axum::{Json, extract::State};
use serde::Deserialize;
use std::sync::{Arc, RwLock};
use crate::map_segment::MapSegment;

#[derive(Deserialize)]
pub struct ScanRequest {
    from_lat: f64,
    from_lon: f64,
    polygon: Vec<(f64, f64)>,
}

pub async fn handler(
    State(ms): State<Arc<RwLock<MapSegment>>>,
    Json(payload): Json<ScanRequest>,
) -> Result<Json<bool>, axum::http::StatusCode> {
    println!("Starting scan...");
    let mut ms = ms.write().map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let pixels = ms.scan_polygon(&payload.polygon);
    let mut count = 0;

    for (x, y, _elev) in pixels {
        let (to_lat, to_lon) = ms.pixel_to_latlon(x, y);
        let has_los = ms.has_line_of_sight(
            payload.from_lat,
            payload.from_lon,
            to_lat,
            to_lon,
            5.0,
            10.0,
        );

        if has_los {
            ms.draw_pixel(x as u32, y as u32);
            count += 1;
        }
    }

    println!("Scan complete: {} pixels with LOS", count);
    Ok(Json(true))
}

