pub mod get_los;
pub mod run_scan;
pub mod get_tile;
pub mod get_tile_v2;
pub mod get_tile_los;
pub mod get_tile_v3;
pub mod get_los_map_img;

use std::sync::{Arc, RwLock};
use crate::world_grid::WorldGrid;
use axum::{
    routing::{get, post},
    Router
};

pub fn router(world: Arc<RwLock<WorldGrid>>) -> Router {
    Router::new()
        .route("/tiles_rf/{z}/{x}/{y}/tile.png", get(get_tile::handler))
        .route("/tiles_los/{z}/{x}/{y}/tile.png", get(get_tile_los::handler))
        .route("/tiles/{z}/{x}/{y}/tile.png", get(get_tile_v2::handler))
        .route("/tile_v3/{z}/{x}/{y}/tile.png", get(get_tile_v3::handler))
        .route("/los", post(get_los::handler))
        .route("/scan", post(run_scan::handler))
        .route("/los_map_img/{idx}", get(get_los_map_img::handler))
        .route("/", axum::routing::get(|| async { "Hello, World!" }))
        .with_state(world)
}

