mod handler;
mod repository;
mod tokenizer;

use crate::handler::{create_tracking_link, resolve_tracking_link};
use crate::repository::{Repository, ShortUrl};
use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use log::info;
use mongodb::Client;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
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
    let repository = Arc::new(Repository::new(collection));

    info!("Starting Actix Web server on 127.0.0.1:8080");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(repository.clone()))
            .configure(configure_routes)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}

fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api/v1")
            .service(web::resource("/shorten").route(web::post().to(create_tracking_link))),
    )
    .service(web::resource("/{token}").route(web::get().to(resolve_tracking_link)));
}
