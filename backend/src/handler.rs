use crate::repository::{Repository, ShortUrl};
use crate::aliaser;
use actix_web::http::header;
use actix_web::{web, HttpResponse};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::sync::Arc;

pub async fn create_shortened_url(
    json: web::Json<ShortenRequest>,
    repo: web::Data<Arc<Repository>>,
) -> HttpResponse {
    let request = json.into_inner();

    let alias = aliaser::generate_alias();
    let short_url = ShortUrl::new(alias.clone(), request.url.clone());
    match repo.insert(short_url).await {
        Ok(_) => {},
        Err(_) => {
            return HttpResponse::InternalServerError().finish()
        },
    };

    let url = format!("{}/{}", request.domain, alias);
    let result = ShortenResponse {
        domain: request.domain,
        alias,
        short_url: url,
    };

    HttpResponse::Created().json(json!({"data": result}))
}

pub async fn resolve_shortened_url(
    path: web::Path<String>,
    repo: web::Data<Arc<Repository>>,
) -> HttpResponse {
    let token = path.into_inner();

    match repo.find_by_token(token).await {
        Ok(short_url) => match short_url {
            Some(short_url) => HttpResponse::PermanentRedirect()
                .insert_header((header::LOCATION, short_url.url))
                .finish(),
            None => HttpResponse::NotFound().finish(),
        },
        Err(_) => HttpResponse::InternalServerError().finish(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ShortenRequest {
    pub domain: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct ShortenResponse {
    pub domain: String,
    pub alias: String,
    pub short_url: String,
}
