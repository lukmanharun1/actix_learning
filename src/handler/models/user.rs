use crate::handler::bin::helper::{ mongo::mongo_async };
use mongodb::{ Collection };
use serde::{Serialize, Deserialize};
use mongodb::bson::oid::ObjectId;

#[derive(Debug, Serialize, Deserialize)]
pub struct User {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<ObjectId>,
    pub username: String,
    pub email: String,
    pub age: u8,
    pub gender: String,
    pub password: String,
}

pub async fn collection_user() -> Collection<User> {
    mongo_async::<User>("users").await
}