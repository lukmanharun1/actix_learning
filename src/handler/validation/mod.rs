use regex::Regex;

pub fn is_email(email: &str) -> bool {
    let email_regex: Regex = Regex::new(r"^([a-z0-9_+]([a-z0-9_+.]*[a-z0-9_+])?)@([a-z0-9]+([\-\.]{1}[a-z0-9]+)*\.[a-z]{2,6})").unwrap();
    email_regex.is_match(email)
}

pub fn is_gender(gender: &str) -> bool {
    if  gender != "male" && gender != "female" {
        return false;
    }
    true
}

pub fn is_password(password: &str) -> Result<bool, String> {
    // min 8 character
    if password.len() < 8 {
        return Err(String::from("Invalid password must be minimum 8 character"));
    }
    // min 1 lowercase
    let mut is_lower:bool = false;
    // min 1 uppercase
    let mut is_upper: bool = false;
    // min 1 number
    let mut is_number: bool = false;
    for word in password.chars() {
        if !is_lower && word.is_lowercase() {
            is_lower = true;
        } else if !is_upper && word.is_uppercase() {
            is_upper = true;
        } else if !is_number && word.is_numeric() {
            is_number = true;
        }
    }
    if !is_lower {
        return Err(String::from("Invalid password must be minimum 1 lowercase"));
    }
    if !is_upper {
        return Err(String::from("Invalid password must be minimum 1 uppercase"));
    }
    if !is_number {
        return Err(String::from("Invalid password must be minimum 1 number"));
    }
    Ok(true)
    
}