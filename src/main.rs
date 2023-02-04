mod handler;

use actix_cors::Cors;
use actix_web::{http, web, App, HttpServer };
use handler::operator::{ auth, profile };

use crate::handler::{ bin::middleware::authentication_token };

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let host: String = dotenv::var("HOST").unwrap_or(String::from("http://localhost"));
    let port: String = dotenv::var("PORT").unwrap_or(String::from("8080"));
    let cors_origin: String = dotenv::var("CORS_ORIGIN").unwrap_or(String::from("http://localhost:3000"));
    println!("starting HTTP server at {}:{}", host, port);
    println!("allowed cors origin {}", cors_origin);
    HttpServer::new(move || {
        let cors = Cors::default()
              .allowed_origin(&cors_origin)
              .allowed_methods(vec!["GET", "POST", "PUT", "PATCH", "DELETE"])
              .allowed_headers(vec![http::header::AUTHORIZATION, http::header::ACCEPT])
              .allowed_header(http::header::CONTENT_TYPE)
              .max_age(3600);
        App::new()
            .wrap(cors)
            .route("/auth", web::post().to(auth::register))
            .route("/auth", web::get().to(auth::login))
            .service(
                web::scope("/profile").wrap(authentication_token::AuthenticationToken)
                    .route("/self", web::get().to(profile::get_profile))
                    .route("/self", web::patch().to(profile::update_profile))
                    .route("/self-image", web::patch().to(profile::update_profile_image))
                    .route("/self", web::delete().to(profile::delete_profile))
            )
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}