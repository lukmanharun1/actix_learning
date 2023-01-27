use serde::{Serialize, Deserialize};

#[derive(Deserialize, Debug)]
pub struct UserRegister {
    pub username: String,
    pub email: String,
    pub age: u8,
    pub gender: String,
    pub password: String
}

#[derive(Deserialize)]
pub struct UserLogin {
    pub email: String,
    pub password: String
}

#[derive(Serialize, Debug)]
pub struct ResponseMessage {
    pub code: i16,
    pub message: String
}

#[derive(Serialize)]
pub struct ResponseResult<Data> {
    pub code: i16,
    pub data: Data
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PayloadUser {
    pub username: String,
    pub email: String,
    pub age: u8,
    pub gender: String,
    pub exp: usize
}


#[derive(Serialize)]
pub struct Token {
    pub token: String
}

#[derive(Serialize)]
pub struct ResponseCreateToken {
    pub code: i16,
    pub data: Token
}

#[derive(Deserialize)]
pub struct UserUpdate {
    pub username: Option<String>,
    pub email: Option<String>,
    pub age: Option<u8>,
    pub gender: Option<String>
}

#[derive(Serialize)]
pub struct ResponseGetProfile {
    pub username: String,
    pub email: String,
    pub age: u8,
    pub gender: String,
}