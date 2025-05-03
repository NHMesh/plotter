use axum::{Json, extract::State};
use serde::Deserialize;
use std::sync::{Arc, RwLock};
//use tracing::info;
use crate::map_segment::MapSegment;

#[derive(Deserialize)]
pub struct LOSRequest {
    from_lat: f64,
    from_lon: f64,
    to_lat: f64,
    to_lon: f64,
    observer_height: Option<f32>,
    target_height: Option<f32>,
}

pub async fn handler(
    State(ms): State<Arc<RwLock<MapSegment>>>,
    Json(payload): Json<LOSRequest>,
) -> Result<Json<bool>, axum::http::StatusCode> {
    let ms = ms.read().map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;

    let observer_height = payload.observer_height.unwrap_or(1.7);
    let target_height = payload.target_height.unwrap_or(1.7);

    let los = ms.has_line_of_sight(
        payload.from_lat,
        payload.from_lon,
        payload.to_lat,
        payload.to_lon,
        observer_height,
        target_height,
    );

    println!("LOS check: from=({}, {}), to=({}, {}), result={}",
        payload.from_lat, payload.from_lon,
        payload.to_lat, payload.to_lon,
        los);

    Ok(Json(los))
}
