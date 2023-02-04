use mongodb::bson::{ doc };
use actix_web::{ web, HttpResponse, Responder};

use crate::handler::{models::{ user::{User, collection_user} }, interface::{ ResponseCreateToken, Token }, validation};
use crate::handler::interface::{ResponseMessage, UserRegister, UserLogin, PayloadUser};
use crate::handler::bin::helper::{ password, jwt };

use mongodb::results::{ InsertOneResult };
use chrono::{Utc, Duration};

pub async fn register(req: web::Json<UserRegister>) -> impl Responder {
    let username: String = req.username.to_string();
    let email: String = req.email.to_string();
    let age: u8 = req.age.into();
    let gender: String = req.gender.to_string(); 
    let password = req.password.as_str();

    // check gender
    if !validation::is_gender(&gender) {
        return  HttpResponse::BadRequest().json(ResponseMessage {
            code: 400,
            message: String::from("Invalid gender must be male or female")
        });
    }
    // check email
    if !validation::is_email(&email) {
        return HttpResponse::BadRequest().json(ResponseMessage {
            code: 400,
            message: String::from("Invalid email")
        });
    }
    // check password
    match validation::is_password(password) {
        Err(err) => return HttpResponse::BadRequest().json(ResponseMessage {
            code: 400,
            message: err
        }),
        Ok(_) => {}
    }
    let password_hash: String = password::hash(password);
    // check username & email unique
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
    let create_user = User {
        id: None,
        username: username,
        email: email,
        age: age,
        gender: gender,
        password: password_hash
    };

    let result: Result<InsertOneResult, _> = collection_user().await.insert_one(create_user, None).await;

    let response = ResponseMessage {
        code: 201,
        message: String::from("Data created successfully")
    };

    match result {
        Ok(_) => HttpResponse::Created().json(response),
        Err(err) => HttpResponse::InternalServerError().json(ResponseMessage {
            code: 500,
            message: err.to_string()
        }),
    }
}

pub async fn login(req: web::Json<UserLogin>) -> impl Responder {
    let email = req.email.to_string();
    let password = req.password.to_string();

    // check email
    if !validation::is_email(&email) {
        return HttpResponse::BadRequest().json(ResponseMessage {
            code: 400,
            message: String::from("Invalid email")
        });
    }
    // check password
    match validation::is_password(&password) {
        Err(err) => return HttpResponse::BadRequest().json(ResponseMessage {
            code: 400,
            message: err
        }),
        Ok(_) => {}
    }

    // check email
    let result = collection_user().await.find_one(doc! {
        "email": email
    }, None).await;

    match result {
        Ok(Some(result)) => {
            // check password 
            if !password::verify(&password, &result.password) {
                return HttpResponse::BadRequest().json(ResponseMessage {
                    code: 400,
                    message: String::from("Incorrect email or password")
                });
            }
            let email: String = result.email.to_string();
            let username: String = result.username.to_string();
            let age: u8 = result.age;
            let gender: String = result.gender.to_string();
            
            let exp: usize = (Utc::now() + Duration::days(1)).timestamp() as usize;

            let payload = PayloadUser {
                exp,
                email: email,
                age: age,
                gender: gender,
                username: username,
                image: None
            };

            let token = jwt::sign::<PayloadUser>(payload);
            return HttpResponse::Ok().json(ResponseCreateToken {
                code: 200,
                data: Token {
                    token
                }
            });
        },
        Ok(None) => {
            return HttpResponse::BadRequest().json(ResponseMessage {
                code: 400,
                message: String::from("Incorrect email or password")
            });
        },
        Err(err) => {
            return HttpResponse::InternalServerError().json(ResponseMessage {
                code: 500,
                message: err.to_string()
            });
        }
    };
}