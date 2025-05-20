// src/web/scan.rs

use axum::{Json, extract::State};
use serde::Deserialize;
use std::sync::{Arc, RwLock};
use crate::map_segment::MapSegment;

#[derive(Deserialize)]
pub struct ScanRequest {
    from_lat: f64,
    from_lon: f64,
    radius: usize,
    observer_height: f32,
}

pub async fn handler(
    State(ms): State<Arc<RwLock<MapSegment>>>,
    Json(payload): Json<ScanRequest>,
) -> Result<Json<bool>, axum::http::StatusCode> {
    println!("Starting LOS map generation...");
    let mut ms = ms.write().map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    ms.generate_los_map(payload.from_lat, payload.from_lon, payload.radius, payload.observer_height);

    println!("LOS map generated for observer at ({}, {}) radius {} height {}", payload.from_lat, payload.from_lon, payload.radius, payload.observer_height);
    Ok(Json(true))
}

