use chrono::Utc;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use mongodb::Collection;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ShortUrl {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub target_url: String,
    pub created_at: i64,
}

impl ShortUrl {
    pub fn new(target_url: String) -> Self {
        ShortUrl {
            id: ObjectId::new(),
            target_url,
            created_at: Utc::now().timestamp(),
        }
    }
}

pub struct Repository {
    collection: Collection<ShortUrl>,
}

impl Repository {
    pub fn new(collection: Collection<ShortUrl>) -> Self {
        Repository { collection }
    }

    pub async fn insert(&self, tracking_link: ShortUrl) -> mongodb::error::Result<ObjectId> {
        let result = self.collection.insert_one(tracking_link).await?;
        Ok(result.inserted_id.as_object_id().unwrap())
    }

    pub async fn find_by_id(&self, id: ObjectId) -> mongodb::error::Result<Option<ShortUrl>> {
        let filter = doc! { "_id": id };
        let result = self.collection.find_one(filter).await?;
        Ok(result)
    }
}
