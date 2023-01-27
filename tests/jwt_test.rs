use actix_learning::handler::bin::helper::jwt;
use actix_learning::handler::interface::PayloadUser;
use chrono::{Utc, Duration};

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn should_create_token_and_verify_token_jwt() {
        let exp: usize = (Utc::now() + Duration::days(1)).timestamp() as usize;
        let username: String = String::from("Lukman");
        let age: u8 = 20;
        let email: String = String::from("lukmanharun925@gmail.com");
        let gender: String = String::from("male");
        let payload = PayloadUser {
            username: username,
            age: age,
            email: email,
            gender: gender,
            exp: exp
        };
        let token = jwt::sign::<PayloadUser>(payload);
        match jwt::verify::<PayloadUser>(token) {
            Err(err) => {println!("failed decode token {}", err.to_string())}
                Ok(decoded_token) => {
                let username: String = String::from("Lukman");
                let age: u8 = 20;
                let email: String = String::from("lukmanharun925@gmail.com");
                let gender: String = String::from("male");
                let exp: usize = (Utc::now() + Duration::days(1)).timestamp() as usize;

                assert_eq!(username, decoded_token.claims.username);
                assert_eq!(age, decoded_token.claims.age);
                assert_eq!(email, decoded_token.claims.email);
                assert_eq!(gender, decoded_token.claims.gender);
                assert_eq!(exp, decoded_token.claims.exp);
            }
        };
    }
}