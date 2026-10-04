use crate::domain::entities::user::{Credentials, Session};
use crate::port::RemoteAuth;

#[derive(Clone)]
pub struct SupabaseAuthService<R: RemoteAuth> {
    remote: R,
}

impl<R: RemoteAuth> SupabaseAuthService<R> {
    pub fn new(remote: R) -> Self {
        Self { remote }
    }

    pub async fn register(&self, creds: &Credentials) -> Result<Session, String> {
        self.remote.signup(creds).await
    }

    pub async fn login(&self, creds: &Credentials) -> Result<Session, String> {
        self.remote.login(creds).await
    }
}
