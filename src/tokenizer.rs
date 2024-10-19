use base64::Engine;
use mongodb::bson::oid::ObjectId;
use xxhash_rust::xxh3::xxh3_64;

pub fn generate_token(db_id: ObjectId, date_created: i64) -> String {
    let hash = produce_hash(&db_id, date_created);
    let raw_token = format!("{}:{}", db_id, hash);
    let base64 = base64::prelude::BASE64_URL_SAFE_NO_PAD.encode(raw_token);
    log::info!("hash: {}, base64: {}", hash, base64);
    base64
}

pub fn produce_hash(db_id: &ObjectId, date_created: i64) -> String {
    let hash_input = format!("{}:{}", db_id.clone(), date_created.clone());
    let hash = xxh3_64(hash_input.as_bytes());
    hash.to_string()
}

pub struct Token {
    pub db_id: String,
    pub hash: String,
}
pub fn parse_token(token: String) -> Result<Token, String> {
    let decoded_bytes = match base64::prelude::BASE64_URL_SAFE_NO_PAD.decode(token) {
        Ok(bytes) => bytes,
        Err(_) => return Err("could not decode token".to_string()),
    };

    let decoded_string = match String::from_utf8(decoded_bytes) {
        Ok(string) => string,
        Err(_) => return Err("could not convert token bytes to string".to_string()),
    };

    let tokens: Vec<&str> = decoded_string.split(':').collect();
    if tokens.len() != 2 {
        return Err("token has incorrect format".to_string());
    }

    let token = Token {
        db_id: tokens[0].to_string(),
        hash: tokens[1].to_string(),
    };
    Ok(token)
}
