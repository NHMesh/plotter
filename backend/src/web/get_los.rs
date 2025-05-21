use axum::{Json, extract::State};
use serde::Deserialize;
use std::sync::{Arc, RwLock};
use crate::world_grid::WorldGrid;

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
    State(world): State<Arc<RwLock<WorldGrid>>>,
    Json(payload): Json<LOSRequest>,
) -> Result<Json<bool>, axum::http::StatusCode> {
    let world = world.read().map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    let observer_height = payload.observer_height.unwrap_or(1.7);
    let target_height = payload.target_height.unwrap_or(1.7);
    // Find the segment containing the observer and the segment containing the target
    let seg_from = world.find_segment(payload.from_lat, payload.from_lon);
    let seg_to = world.find_segment(payload.to_lat, payload.to_lon);
    if let (Some(seg_from), Some(seg_to)) = (seg_from, seg_to) {
        // If both points are in the same segment, use the segment's LOS
        if std::ptr::eq(seg_from, seg_to) {
            let los = seg_from.has_line_of_sight(
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
        } else {
            // TODO: Implement multi-segment LOS (across segment boundaries)
            println!("LOS check: from=({}, {}), to=({}, {}), result=UNSUPPORTED (multi-segment)",
                payload.from_lat, payload.from_lon,
                payload.to_lat, payload.to_lon);
            Err(axum::http::StatusCode::NOT_IMPLEMENTED)
        }
    } else {
        Err(axum::http::StatusCode::NOT_FOUND)
    }
}
