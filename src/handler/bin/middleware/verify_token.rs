use actix_web::{ HttpResponse, HttpRequest};
use crate::handler::bin::helper::jwt;
pub fn verify_token(req: &HttpRequest) -> HttpResponse {
    let token : String = req.headers().get("Authorization")?.to_str().unwrap().to_string();
    jwt::verify(token);
    // if token.is_some() {
    //     jwt::verify(token);
    // }
    // jwt::verify(token)
}