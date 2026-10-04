use crate::domain::entities::user::{
    Credentials, ERR_INVALID_CREDENTIALS, ERR_USERNAME_TAKEN, Session,
};
use crate::port::{PasswordHasher, TokenIssuer, UserStore};

#[derive(Clone)]
pub struct AuthService<S: UserStore, H: PasswordHasher, T: TokenIssuer> {
    store: S,
    hasher: H,
    tokens: T,
}

impl<S: UserStore, H: PasswordHasher, T: TokenIssuer> AuthService<S, H, T> {
    pub fn new(store: S, hasher: H, tokens: T) -> Self {
        Self {
            store,
            hasher,
            tokens,
        }
    }

    pub async fn register(&self, creds: &Credentials) -> Result<Session, String> {
        if self.store.exists(&creds.username).await? {
            return Err(ERR_USERNAME_TAKEN.to_string());
        }
        let hash = self.hasher.hash(&creds.password)?;
        let username = self.store.create(&creds.username, &hash).await?;
        Ok(Session {
            token: self.tokens.issue(),
            username,
        })
    }

    pub async fn login(&self, creds: &Credentials) -> Result<Session, String> {
        let hash = self.store.password_hash(&creds.username).await?;
        let ok = hash
            .as_deref()
            .map(|h| self.hasher.verify(&creds.password, h))
            .unwrap_or(false);
        if !ok {
            return Err(ERR_INVALID_CREDENTIALS.to_string());
        }
        Ok(Session {
            token: self.tokens.issue(),
            username: creds.username.clone(),
        })
    }
}
