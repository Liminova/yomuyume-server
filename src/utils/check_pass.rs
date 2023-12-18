use argon2::{Argon2, PasswordHash, PasswordVerifier};

pub fn check_pass(real: &str, input: &String) -> bool {
    match PasswordHash::new(real) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(input.as_bytes(), &parsed_hash)
            .map_or(false, |_| true),
        Err(_) => false,
    }
}
