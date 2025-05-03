// src/main.rs

mod map_segment;
mod web;

use std::sync::{Arc, RwLock};
use tower_http::cors::CorsLayer;
//use tracing_subscriber;
use map_segment::MapSegment;
use web::router;

#[tokio::main]
async fn main() {
    // Set up logging
    //tracing_subscriber::fmt::init();

    // Load DEM dataset and generate the base image
    let mut ms = MapSegment::load_from_dataset("USGS_13_n44w072_20240617.tif")
        .expect("Failed to load dataset");
    ms.gen_map().expect("Failed to generate map");

    let ms = Arc::new(RwLock::new(ms));

    // Build the application router
    let app = router(ms).layer(CorsLayer::permissive());

    // Start the Axum server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind listener");

    axum::serve(listener, app).await.unwrap();
}
