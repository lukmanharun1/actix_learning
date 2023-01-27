use serde::{ Serialize, de::DeserializeOwned};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, TokenData, Validation, errors::Error };

pub fn sign<T: Serialize>(payload: T) -> String {
    // my_claims is a struct that implements Serialize
    // This will create a JWT using HS256 as algorithm
    encode(&Header::default(), &payload, &EncodingKey::from_secret(dotenv::var("JWT_TOKEN_SECRET").unwrap().as_ref())).unwrap()
}

pub fn verify<T: DeserializeOwned>(token: String) -> Result<TokenData<T>, Error> {
    // `token` is a struct with 2 fields: `header` and `claims` where `claims` is your own struct.
    decode::<T>(&token, &DecodingKey::from_secret(dotenv::var("JWT_TOKEN_SECRET").unwrap().as_ref()), &Validation::default())
}