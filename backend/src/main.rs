mod handler;
mod repository;
mod aliaser;

use crate::handler::{create_shortened_url, resolve_shortened_url};
use crate::repository::{Repository, ShortUrl};
use actix_web::{web, App, HttpServer};
use dotenvy::dotenv;
use mongodb::Client;
use std::sync::Arc;
use actix_cors::Cors;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // load environment stuff
    dotenv().ok();

    // init logger
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info) // Set the log level to INFO
        .init();

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
        Err(error) => panic!("failed to init repo: {:?}", error)
    };

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
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
            .service(web::resource("/shorten").route(web::post().to(create_shortened_url))),
    )
    .service(web::resource("/{token}").route(web::get().to(resolve_shortened_url)));
}
