use crate::aliaser;
use crate::repository::{Repository, ShortUrl};

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{header, StatusCode};
use axum::Json;
use axum::response::{Response};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub async fn create_shortened_url(
    State(repo): State<Arc<Repository>>,
    Json(request): Json<ShortenRequest>,
) -> Response {
    let alias = aliaser::generate_alias();
    let short_url = ShortUrl::new(alias.clone(), request.url.clone());
    match repo.insert(short_url).await {
        Ok(_) => {
            let url = format!("{}/{}", request.domain, alias);
            let result = ShortenDataResponse {
                data: ShortUrlResponse {
                    domain: request.domain,
                    alias,
                    short_url: url,
                }
            };
            let json_body = serde_json::to_string(&result).unwrap();
            Response::builder()
                .status(StatusCode::CREATED)
                .body(Body::from(json_body))
                .unwrap()
        }
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap()
    }
}

pub async fn resolve_shortened_url(
    State(repo): State<Arc<Repository>>,
    Path(token): Path<String>,
) -> Response {
    println!("token: {}", format!("{}", token));
    match repo.find_by_token(token).await {
        Ok(short_url) => match short_url {
            Some(short_url) => Response::builder()
                .status(StatusCode::MOVED_PERMANENTLY)
                .header(header::LOCATION, &short_url.url)
                .body(Body::empty())
                .unwrap(),
            None => Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .unwrap(),
        },
        Err(_) => Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .unwrap(),
    }
}

#[derive(Debug, Deserialize)]
pub struct ShortenRequest {
    pub domain: String,
    pub url: String,
}

#[derive(Debug, Serialize)]
pub struct ShortUrlResponse {
    pub domain: String,
    pub alias: String,
    pub short_url: String,
}

#[derive(Debug, Serialize)]
pub struct ShortenDataResponse {
    pub data: ShortUrlResponse,
}
