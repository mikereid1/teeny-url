mod aliaser;
mod handler;
mod repository;

use crate::handler::{create_shortened_url, resolve_shortened_url};
use crate::repository::{Repository, ShortUrl};
use axum::routing::{get, post};
use axum::Router;
use dotenvy::dotenv;
use mongodb::Client;
use std::sync::Arc;
use tower_http::cors::CorsLayer;

#[tokio::main]
async fn main() {
    // load environment stuff
    dotenv().ok();

    // get mongo connection env variables
    let mongo_uri = std::env::var("MONGODB_URI").expect("missing environment variable MONGODB_URI");
    let db_name =
        std::env::var("MONGODB_DATABASE").expect("missing environment variable MONGODB_DATABASE");
    let collection_name = std::env::var("MONGODB_COLLECTION")
        .expect("missing environment variable MONGODB_COLLECTION");

    // connect to mongo
    let database = Client::with_uri_str(&mongo_uri)
        .await
        .expect("failed to initialise client")
        .database(&db_name);

    // setup repository
    let collection = database.collection::<ShortUrl>(&collection_name);
    let repository = match Repository::new(collection).await {
        Ok(repository) => Arc::new(repository),
        Err(error) => panic!("failed to init repo: {:?}", error),
    };

    // handle CORS
    let cors = CorsLayer::very_permissive();

    // setup routes
    let app = Router::new()
        .route("/api/v1/shorten", post(create_shortened_url))
        .route("/:token", get(resolve_shortened_url))
        .with_state(repository.clone())
        .layer(cors);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}
