use serde::{Deserialize, Serialize};

pub const ERR_USERNAME_TAKEN: &str = "username already taken";
pub const ERR_INVALID_CREDENTIALS: &str = "invalid username or password";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Credentials {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub token: String,
    pub username: String,
}
