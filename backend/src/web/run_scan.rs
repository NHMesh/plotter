// src/web/scan.rs

use axum::{Json, extract::State};
use serde::Deserialize;
use std::sync::{Arc, RwLock};
use crate::world_grid::WorldGrid;
use crate::map_segment::MapSegment;

#[derive(Deserialize)]
pub struct ScanRequest {
    from_lat: f64,
    from_lon: f64,
    radius: usize,
    observer_height: f32,
}

pub async fn handler(
    State(world): State<Arc<RwLock<WorldGrid>>>,
    Json(payload): Json<ScanRequest>,
) -> Result<Json<bool>, axum::http::StatusCode> {
    println!("Starting LOS map generation...");
    let observer_lat = payload.from_lat;
    let observer_lon = payload.from_lon;
    let radius = payload.radius;
    let observer_height = payload.observer_height;

    // Lock for reading to check if observer is in a segment
    {
        let world = world.read().map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
        if world.find_segment(observer_lat, observer_lon).is_none() {
            return Err(axum::http::StatusCode::NOT_FOUND);
        }
    }

    // Generate and add the LOS map with a write lock
    {
        let mut world = world.write().map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
        world.generate_and_add_los_map(
            observer_lat,
            observer_lon,
            radius,
            observer_height,
        );
    }
    println!("LOS map generated for observer at ({}, {}) radius {} height {}", observer_lat, observer_lon, radius, observer_height);
    Ok(Json(true))
}

