use actix_web::{ web, web::{ReqData}, HttpResponse, Responder, Error};
use mongodb::bson::{doc, Bson};
use std::io::Write;
use std::fs::{self as fs, File};
use crate::handler::{models::{ user::{ collection_user} }, interface::{ UserUpdate, ResponseCreateToken, Token, ResponseGetProfile }, validation, bin::helper::random};
use crate::handler::interface::{ResponseMessage, ResponseResult, PayloadUser};
use actix_multipart::Multipart;
use futures_util::{TryStreamExt as _};
use image;
use bytes::BytesMut;
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


pub async fn update_profile(decoded_token: ReqData<PayloadUser>,  req: web::Json<UserUpdate>) -> impl Responder {
    let username = &req.username;
    let email = &req.email;
    let age = &req.age;
    let gender = &req.gender;

   let mut payload = PayloadUser {
        exp: decoded_token.exp as usize,
        email: decoded_token.email.to_string(),
        age: decoded_token.age,
        gender: decoded_token.gender.to_string(),
        username: decoded_token.username.to_string(),
        image: None
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


pub async fn update_profile_image(decoded_token: ReqData<PayloadUser>, mut payload: Multipart) -> Result<impl Responder, Error> {
    // iterate over multipart stream
    while let Some(mut field) = payload.try_next().await? {
        // A multipart/form-data stream has to contain `content_disposition`
        let content_disposition = field.content_disposition().clone();
        match content_disposition.get_filename() {
            Some(_) => {
                match content_disposition.get_name() {
                    Some(key) => {
                        if key != "image" {
                            return Ok(HttpResponse::BadRequest().json(ResponseMessage {
                                code: 400,
                                message: String::from("Invalid key must be image!")
                            }));
                        }
                    },
                    None => {
                        return Ok(HttpResponse::BadRequest().json(ResponseMessage {
                            code: 400,
                            message: String::from("Invalid key must be image!")
                        }));
                    }
                };

                let max_size_image = 1024 * 1024 * 2; // 2MB
                let mut chunks = BytesMut::with_capacity(max_size_image);
                while let Some(chunk) = field.try_next().await? {
                    chunks.extend_from_slice(&chunk);
                }

                if chunks.len() > max_size_image {
                    return Ok(HttpResponse::BadRequest().json(ResponseMessage {
                        code: 400,
                        message: String::from("Image size must be less than 2MB!")
                    }));
                }
                // check image extension
                match image::guess_format(&chunks) {
                    Ok(image_format) => {
                        let extension = image_format.extensions_str()[0];

                        let extensions = vec!["jpg", "png", "webp"];
                        if !extensions.contains(&extension) {
                            return Ok(HttpResponse::BadRequest().json(ResponseMessage {
                                code: 400,
                                message: String::from("Invalid image format!")
                            }));
                        }
                        let filename = format!("{}.{}", random::strings(24), extension);
                        // upload image
                        let filepath = format!("./images/{filename}");
                        // File::create is blocking operation, use threadpool
                        let mut f = web::block(|| File::create(filepath)).await??;
                        web::block(move || f.write_all(&chunks).map(|_| f)).await??;

                        // update image profile
                        let update_user = collection_user().await.update_one(doc! {
                            "username": decoded_token.username.to_string(),
                            "email": decoded_token.email.to_string()
                        }, doc! {
                            "$set": doc! {
                                "image": &filename
                            }
                        }, None).await;
                        match update_user {
                            Ok(update_result) => {
                                println!("update result: {:?}", update_result);
                                if update_result.modified_count > 0 {
                                    // update jwt token
                                    let payload_user = PayloadUser {
                                        exp: decoded_token.exp as usize,
                                        email: decoded_token.email.to_string(),
                                        age: decoded_token.age,
                                        gender: decoded_token.gender.to_string(),
                                        username: decoded_token.username.to_string(),
                                        image: Some(filename)
                                    };
                                    // delete image
                                    println!("image: {:?}", decoded_token.image);
                                    match &decoded_token.image {
                                        Some(image) => {
                                            let path = "./images/".to_string() + image;
                                            println!("path: {}", path);
                                            match web::block(|| fs::remove_file(path)).await {
                                                Ok(_) => {},
                                                Err(_) => {}
                                            };
                                        }
                                        None => {}
                                    }
                                    let token = jwt::sign(payload_user);
                                    return Ok(HttpResponse::Ok().json(ResponseCreateToken {
                                        code: 200,
                                        data: Token {
                                            token
                                        }
                                    }));
                                }
                                // user sedang mencoba update token yang lama
                                return Ok(HttpResponse::BadRequest().json(ResponseMessage {
                                    code: 400,
                                    message: String::from("Token expired")
                                }));
                            }
                            Err(err) => {
                                return Ok(HttpResponse::InternalServerError().json(ResponseMessage {
                                    code: 500,
                                    message: err.to_string()
                                }));
                            }
                        }
                    },
                    Err(_) => return Ok(
                                HttpResponse::BadRequest().json(ResponseMessage {
                                code: 400,
                                message: String::from("Invalid image format!")
                            })
                        )
                };
            },
            None => {
                return Ok(HttpResponse::BadRequest().json(ResponseMessage {
                    code: 400,
                    message: String::from("Image is required")
                }));
            }
        }
    }

    Ok(HttpResponse::Ok().json(ResponseMessage {
        code: 200,
        message: String::from("Image uploaded successfully")
    }))
}