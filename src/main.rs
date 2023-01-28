mod handler;

use actix_web::{ web, App, HttpServer };
use handler::operator::{ auth, profile };
use crate::handler::{ bin::middleware::authentication_token };

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    let host: String = dotenv::var("HOST").unwrap_or("http://localhost".to_string());
    let port: String = dotenv::var("PORT").unwrap_or("8080".to_string());

    println!("starting HTTP server at {}:{}", host, port);
    HttpServer::new(move || {
        App::new()
            .route("/auth", web::post().to(auth::register))
            .route("/auth", web::get().to(auth::login))
            .service(
                web::scope("/profile").wrap(authentication_token::AuthenticationToken)
                    .route("/self", web::get().to(profile::get_profile))
                    .route("/self", web::patch().to(profile::update_profile))
                    .route("/self", web::delete().to(profile::delete_profile))
            )
                   
    })
    .bind(format!("{}:{}", host, port))?
    .run()
    .await
}