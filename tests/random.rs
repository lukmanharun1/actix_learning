use actix_learning::handler::bin::helper::random;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_create_random_string_success() {
        let random_string = random::strings(10);
        assert_eq!(random_string.len(), 10);
    }
}
