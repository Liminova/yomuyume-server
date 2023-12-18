use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub enum ConfirmReason {
    Register,
    Forget,
    Delete,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConfirmTokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
    pub reason: ConfirmReason,
}
