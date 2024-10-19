use crate::repository::{Repository, ShortUrl};
use crate::tokenizer;
use actix_web::http::header;
use actix_web::{web, HttpResponse};
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::str::FromStr;
use std::sync::Arc;

pub async fn create_tracking_link(
    json: web::Json<ShortenRequest>,
    repo: web::Data<Arc<Repository>>,
) -> HttpResponse {
    let request = json.into_inner();

    let short_url = ShortUrl::new(request.target_url.clone());
    let db_id = match repo.insert(short_url).await {
        Ok(id) => id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };
    let created_at = match repo.find_by_id(db_id).await {
        Ok(short_url) => match short_url {
            Some(short_url) => short_url.created_at,
            None => return HttpResponse::InternalServerError().finish(),
        },
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    let token = tokenizer::generate_token(db_id, created_at);

    let url = format!("{}/{}", request.domain, token);
    let result = ShortenResponse {
        domain: request.domain,
        alias: token,
        short_url: url,
    };

    HttpResponse::Created().json(json!({"data": result}))
}

pub async fn resolve_tracking_link(
    path: web::Path<String>,
    repo: web::Data<Arc<Repository>>,
) -> HttpResponse {
    let token = path.into_inner();

    // parse token
    let token = match tokenizer::parse_token(token) {
        Ok(token) => token,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // parse db_id
    let db_id = match ObjectId::from_str(&token.db_id) {
        Ok(id) => id,
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // find db entry
    let short_url = match repo.find_by_id(db_id).await {
        Ok(short_url) => match short_url {
            Some(short_url) => short_url,
            None => return HttpResponse::NotFound().finish(),
        },
        Err(_) => return HttpResponse::InternalServerError().finish(),
    };

    // verify hash
    let hash = tokenizer::produce_hash(&db_id, short_url.created_at);
    match token.hash == hash {
        true => HttpResponse::PermanentRedirect()
            .insert_header((header::LOCATION, short_url.target_url))
            .finish(),
        false => HttpResponse::NotFound().finish(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ShortenRequest {
    pub domain: String,
    pub target_url: String,
}

#[derive(Debug, Serialize)]
pub struct ShortenResponse {
    pub domain: String,
    pub alias: String,
    pub short_url: String,
}
