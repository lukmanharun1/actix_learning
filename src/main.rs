mod handler;

use actix_web::{ web, web::ReqData, App, HttpResponse, HttpServer, Responder};
use mongodb::bson::{doc, Bson};
use handler::{models::{ user::{User, collection_user} }, interface::{ UserUpdate, ResponseCreateToken, Token, ResponseGetProfile }, validation};
use handler::interface::{ResponseMessage, ResponseResult, UserRegister, UserLogin, PayloadUser};
use handler::bin::helper::{ password, jwt };
use mongodb::results::{ InsertOneResult };
use chrono::{Utc, Duration};

use crate::handler::{ bin::middleware::authentication_token };

async fn register(req: web::Json<UserRegister>) -> impl Responder {
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


async fn login(req: web::Json<UserLogin>) -> impl Responder {
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
                username: username
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

async fn get_profile(decoded_token: ReqData<PayloadUser>) -> impl Responder {
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

async fn update_profile(decoded_token: ReqData<PayloadUser>, req: web::Json<UserUpdate>) -> impl Responder {
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

async fn delete_profile(decoded_token: ReqData<PayloadUser>) -> impl Responder {
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


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let host: String = dotenv::var("HOST").unwrap_or("http://localhost".to_string());
    let port: String = dotenv::var("PORT").unwrap_or("8080".to_string());

    println!("starting HTTP server at {}:{}", host, port);
    HttpServer::new(move || {
        App::new()
            .route("/auth", web::post().to(register))
            .route("/auth", web::get().to(login))
            .service(
                web::scope("/profile").wrap(authentication_token::AuthenticationToken)
                    .route("/self", web::get().to(get_profile))
                    .route("/self", web::patch().to(update_profile))
                    .route("/self", web::delete().to(delete_profile))
            )
                   
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}