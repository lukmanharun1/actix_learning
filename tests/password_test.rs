use actix_learning::handler::bin::helper::password;


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_and_verify_password_success() {
        let password = "Lukman@password";
        let password_hash = password::hash(password);
        assert_eq!(password::verify(password, &password_hash), true);
    }
    #[test]
    fn should_create_and_verify_password_failed() {
        let password = "Lukman@password";
        let password_hash = password::hash(password);
        assert_eq!(password::verify("password failed", &password_hash), false);
    }

}