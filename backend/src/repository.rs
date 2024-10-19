use chrono::Utc;
use mongodb::bson::doc;
use mongodb::bson::oid::ObjectId;
use mongodb::{Collection, IndexModel};
use mongodb::options::IndexOptions;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ShortUrl {
    #[serde(rename = "_id")]
    pub id: ObjectId,
    pub alias: String,
    pub url: String,
    pub created_at: i64,
}

impl ShortUrl {
    pub fn new(alias: String, target_url: String) -> Self {
        ShortUrl {
            id: ObjectId::new(),
            alias,
            url: target_url,
            created_at: Utc::now().timestamp(),
        }
    }
}

pub struct Repository {
    collection: Collection<ShortUrl>,
}

impl Repository {
    pub async fn new(collection: Collection<ShortUrl>) -> mongodb::error::Result<Repository> {
        let index = IndexModel::builder()
            .keys(doc! {"alias": 1})
            .options(Some(IndexOptions::builder().unique(true).build()))
            .build();
        collection.create_index(index).await?;

        Ok(Repository { collection })
    }

    pub async fn insert(&self, short_url: ShortUrl) -> mongodb::error::Result<ObjectId> {
        let result = self.collection.insert_one(short_url).await?;
        Ok(result.inserted_id.as_object_id().unwrap())
    }

    pub async fn find_by_token(&self, alias: String) -> mongodb::error::Result<Option<ShortUrl>> {
        let filter = doc! { "alias": alias };
        let result = self.collection.find_one(filter).await?;
        Ok(result)
    }
}
