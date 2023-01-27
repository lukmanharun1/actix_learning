use sha_crypt::{Sha512Params, sha512_simple, sha512_check};

pub fn hash(password: &str) -> String {
    let params = Sha512Params::new(10_00).expect("RandomError!");
    return sha512_simple(password, &params).expect("Should not fail");
}
pub fn verify(password: &str, hashed_password: &str) -> bool {
    sha512_check(password, hashed_password).is_ok()
}