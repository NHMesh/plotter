pub mod get_los;
pub mod run_scan;
pub mod get_tile;

use std::sync::{Arc, RwLock};
use crate::map_segment::MapSegment;
use axum::{
    routing::{get, post},
    Router
};

pub fn router(ms: Arc<RwLock<MapSegment>>) -> Router {
    Router::new()
        .route("/tiles/{z}/{x}/{y}/tile.png", get(get_tile::handler))
        .route("/los", post(get_los::handler))
        .route("/scan", post(run_scan::handler))
        .route("/", axum::routing::get(|| async { "Hello, World!" }))
        .with_state(ms)
}

