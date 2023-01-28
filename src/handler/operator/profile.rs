use actix_web::{ web, web::ReqData, HttpResponse, Responder};
use mongodb::bson::{doc, Bson};
use crate::handler::{models::{ user::{ collection_user} }, interface::{ UserUpdate, ResponseCreateToken, Token, ResponseGetProfile }, validation};
use crate::handler::interface::{ResponseMessage, ResponseResult, PayloadUser};
use crate::handler::bin::helper::{ jwt };

pub async fn get_profile(decoded_token: ReqData<PayloadUser>) -> impl Responder {
    let username = decoded_token.username.to_string();
    let age = decoded_token.age;
    let email = decoded_token.email.to_string();
    let gender = decoded_token.gender.to_string();

    let user = collection_user().await.find_one(doc! {
            "$and": vec! [
                doc! { "username": username.clone() },
                doc! { "email": email.clone() },
                doc! { "age": Bson::Int32(age.clone().into()) },
                doc! { "gender": gender.clone() },

            ]
        }, None).await;

    match user {
        Ok(Some(_)) => {
            HttpResponse::Ok().json(ResponseResult {
                code: 200,
                data: ResponseGetProfile {
                    username,
                    age,
                    email,
                    gender
                }
            })      
        },
        Ok(None) => {
            return HttpResponse::BadRequest().json(ResponseMessage {
                code: 400,
                message: String::from("Token expired")
            });
        },
        Err(err) => {
            return HttpResponse::InternalServerError().json(ResponseMessage {
                code: 500,
                message: err.to_string()
            });
        }
    }
}


pub async fn update_profile(decoded_token: ReqData<PayloadUser>, req: web::Json<UserUpdate>) -> impl Responder {
    let username = &req.username;
    let email = &req.email;
    let age = &req.age;
    let gender = &req.gender;

   let mut payload = PayloadUser {
        exp: decoded_token.exp as usize,
        email: decoded_token.email.to_string(),
        age: decoded_token.age,
        gender: decoded_token.gender.to_string(),
        username: decoded_token.username.to_string()
    };
    // update data
    let mut data_update = doc! {};
    match username {
        Some(username) => {
            data_update.insert("username", username);
            payload.username = username.to_string();
        }
        None => {}
    }
    match email {
        Some(email) => {
            // check email
            if !validation::is_email(email) {
                return HttpResponse::BadRequest().json(ResponseMessage {
                    code: 400,
                    message: String::from("Invalid email")
                });
            }
            data_update.insert("email", email);
            payload.email = email.to_string();
        }
        None => {}
    }
    match age {
        Some(age) => {
            data_update.insert("age", Bson::Int32(age.clone().into()));
            payload.age = *age;
        }
        None => {}
    }
    match gender {
        Some(gender) => {
            // check gender
            if !validation::is_gender(&gender) {
                return  HttpResponse::BadRequest().json(ResponseMessage {
                    code: 400,
                    message: String::from("Invalid gender must be male or female")
                });
            }
            data_update.insert("gender", gender);
            payload.gender = gender.to_string();
        },
        None => {}
    }
    // check username & email unique
    if username.is_some() || email.is_some() {
        let user = collection_user().await.find_one(doc! {
            "$or": vec! [
                doc! { "username": &username },
                doc! { "email": &email }
            ]
        }, None).await;

        match user {
            Ok(Some(_)) => {
                return HttpResponse::BadRequest().json(ResponseMessage {
                    code: 400,
                    message: String::from("Data user already exists!")
                });
            },
            Ok(None) => {},
            Err(err) => {
                HttpResponse::InternalServerError().json(ResponseMessage {
                    code: 500,
                    message: err.to_string()
                });
            }
        };
    }
    let update_user = collection_user().await.update_one(doc! {
        "username": decoded_token.username.to_string(),
        "email": decoded_token.email.to_string()
    }, doc! {
        "$set": data_update
    }, None).await;

    match update_user {
        Ok(update_result) => {
            if update_result.modified_count > 0 {
                // update token
                let token = jwt::sign(payload);
                return HttpResponse::Ok().json(ResponseCreateToken {
                    code: 200,
                    data: Token {
                        token
                    }
                });
            }
            // user sedang mencoba update token yang lama
            return HttpResponse::BadRequest().json(ResponseMessage {
                code: 400,
                message: String::from("Token expired")
            });
        }
        Err(err) => {
            return  HttpResponse::InternalServerError().json(ResponseMessage {
                code: 500,
                message: err.to_string()
            });
        }
    }
}


pub async fn delete_profile(decoded_token: ReqData<PayloadUser>) -> impl Responder {
    let username = decoded_token.username.to_string();
    let email = decoded_token.email.to_string();

    // delete profile
    let delete_result = collection_user().await.delete_one(doc! {
        "username": username,
        "email": email,
    }, None).await;
    match delete_result {
        Ok(delete_result) => {
            if delete_result.deleted_count > 0 {
                return HttpResponse::Ok().json(ResponseMessage {
                    code: 200,
                    message: String::from("Data profile deleted successfully")
                });
            }
        }
        Err(err) => {
            return  HttpResponse::InternalServerError().json(ResponseMessage {
                code: 500,
                message: err.to_string()
            });
        }
    }
    // user sedang mencoba delete 2 kali
    return HttpResponse::BadRequest().json(ResponseMessage {
        code: 400,
        message: String::from("Token expired")
    });
}
