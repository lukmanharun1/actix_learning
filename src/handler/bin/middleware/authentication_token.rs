use std::{future::{ready, Ready}};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, HttpMessage, HttpResponse, body::{ EitherBody}
};
use futures_util::future::LocalBoxFuture;

use crate::handler::{interface::{ResponseMessage, PayloadUser}, bin::helper::jwt};

// There are two steps in middleware processing.
// 1. Middleware initialization, middleware factory gets called with
//    next service in chain as parameter.
// 2. Middleware's call method gets called with normal request.
#[derive(Debug, Clone)]
pub struct AuthenticationToken;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for AuthenticationToken
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type InitError = ();
    type Transform = AuthenticationTokenMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthenticationTokenMiddleware { service }))
    }

}

pub struct AuthenticationTokenMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for AuthenticationTokenMiddleware<S> 
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<EitherBody<B>>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, service_request: ServiceRequest) -> Self::Future {
        match service_request.request().headers().get("Authorization") {
            None => {
               let response = HttpResponse::Unauthorized().json(ResponseMessage { code: 400, message: String::from("Invalid token!") }).map_into_right_body();
                // constructed responses map to "right" body
                return Box::pin(async move { Ok(ServiceResponse::new(service_request.request().clone(), response)) });
            }
            Some(token) => {
                // validation token
                let decoded_token = jwt::verify::<PayloadUser>(token.to_str().unwrap().to_string());
                match decoded_token {
                    Err(err) => {
                        let response = HttpResponse::Unauthorized().json(ResponseMessage { code: 400, message: err.to_string() }).map_into_right_body();
                        return Box::pin(async move { Ok(ServiceResponse::new(service_request.request().clone(), response)) });
                    }
                    Ok(decoded_token) => {
                        let username = decoded_token.claims.username;
                        let email = decoded_token.claims.email;
                        let age = decoded_token.claims.age;
                        let exp = decoded_token.claims.exp;
                        let gender = decoded_token.claims.gender;
                        let image = decoded_token.claims.image;
                        service_request.extensions_mut().insert::<PayloadUser>(PayloadUser {
                            username,
                            age,
                            email,
                            exp,
                            gender,
                            image
                        });
                    }
                }
            }
        }
        let res = self.service.call(service_request);
        return Box::pin(async move {
            Ok(res.await?.map_into_left_body())
        });
    }
}