use actix_learning::handler::validation;


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_validation_email_success() {
        let email1 = "lukman@gmail.com";
        let email2 = "harun@mail.com";

        assert_eq!(validation::is_email(email1), true);
        assert_eq!(validation::is_email(email2), true);
    }
    #[test]
    fn should_validation_email_failed() {
        let emails = vec!["lukman", "harun@gmail", "lukman.com", "lukman@harun.c"];
        for email in emails {
            assert_eq!(validation::is_email(email), false);
        }
    }

    #[test]
    fn should_validation_gender_success() {
        let male = "male";
        let female = "female";
        assert_eq!(validation::is_gender(male), true);
        assert_eq!(validation::is_gender(female), true);
    }

    #[test]
    fn should_validation_gender_failed() {
        let gender_in_thailands = vec!["MALE", "FEMALE", "Tom", "Dee", "Tom Gay", "Tom Gay King", "Bi", "Boat", "Gay Queen", "Gay King", "Tom Gay Queen", "Tom Gay Two Way", "Lesbian", "Lady Boy", "Adam", "Angee", "Cherry", "Samyaan"];

        for gender in gender_in_thailands {
            assert_eq!(validation::is_gender(gender), false);
        }
    }

    #[test]
    fn should_validation_password_success() {
        assert_eq!(validation::is_password("Harunlukman1").unwrap(), true);
    }

    #[test]
    fn should_validation_password_failed_invalid_password_must_be_minimum_8_character() {
        match validation::is_password("1234567") {
            Err(err) => {
                assert_eq!(err, String::from("Invalid password must be minimum 8 character"));
            },
            Ok(_) => {}
        }
    }
    #[test]
    fn should_validation_password_failed_invalid_password_must_be_minimum_1_lowercase() {
        match validation::is_password("UPPRCASE1") {
            Err(err) => {
                assert_eq!(err, String::from("Invalid password must be minimum 1 lowercase"));
            },
            Ok(_) => {}
        }
    }

    #[test]
    fn should_validation_password_failed_invalid_password_must_be_minimum_1_uppercase() {
        match validation::is_password("lowercase1") {
            Err(err) => {
                assert_eq!(err, String::from("Invalid password must be minimum 1 uppercase"));
            },
            Ok(_) => {}
        }
    }

    #[test]
    fn should_validation_password_failed_invalid_password_must_be_minimum_1_number() {
        match validation::is_password("lowercaseUPPERCASE") {
            Err(err) => {
                assert_eq!(err, String::from("Invalid password must be minimum 1 number"));
            },
            Ok(_) => {}
        }
    }
}