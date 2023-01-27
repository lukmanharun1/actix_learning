use dotenv;
use mongodb::{ Client, Collection };

pub async fn mongo_async<T>(collection: &str) -> Collection<T> {
    let mongodb_uri = dotenv::var("MONGODB_URI").unwrap();
    let db_name = dotenv::var("DB_NAME").unwrap();
    let client = Client::with_uri_str(mongodb_uri).await.expect("failed to connect");
    return client.database(&db_name).collection::<T>(collection);
}