// src/main.rs

mod map_segment;
mod web;
mod los_map;
mod world_grid;

use std::sync::{Arc, RwLock};
use tower_http::cors::CorsLayer;
//use tracing_subscriber;
use web::router;
use crate::world_grid::WorldGrid;

#[tokio::main]
async fn main() {
    // Set up logging
    //tracing_subscriber::fmt::init();

    // Load DEM datasets and generate the base images
    let mut world = WorldGrid::new();
    world.load_segment("USGS_13_n44w072_20240617.tif").expect("Failed to load segment 1");
    world.load_segment("USGS_13_n43w072_20240130.tif").expect("Failed to load segment 2");

    let world = Arc::new(RwLock::new(world));

    // Build the application router
    let app = router(world).layer(CorsLayer::permissive());

    // Start the Axum server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind listener");

    axum::serve(listener, app).await.unwrap();
}
