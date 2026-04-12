use crate::models::Session;
use serde::{Deserialize, Serialize};

/// Session storage for managing auth sessions in localStorage
#[derive(Clone)]
pub struct SessionStorage;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct StoredSession {
    session: Session,
}

impl SessionStorage {
    const SESSION_KEY: &str = "cms_auth_session";

    /// Saves the session to localStorage
    pub fn save_session(session: &Session) -> Result<(), String> {
        if let Some(window) = web_sys::window()
            && let Ok(Some(storage)) = window.local_storage()
        {
            let stored = StoredSession {
                session: session.clone(),
            };
            let json = serde_json::to_string(&stored)
                .map_err(|e| format!("Failed to serialize session: {}", e))?;
            storage
                .set_item(Self::SESSION_KEY, &json)
                .map_err(|e| format!("Failed to save session: {:?}", e))?;
            return Ok(());
        }
        Err("Failed to access localStorage".to_string())
    }

    /// Loads the session from localStorage
    pub fn load_session() -> Result<Option<Session>, String> {
        if let Some(window) = web_sys::window()
            && let Ok(Some(storage)) = window.local_storage()
            && let Ok(Some(json)) = storage.get_item(Self::SESSION_KEY)
        {
            let stored: StoredSession = serde_json::from_str(&json)
                .map_err(|e| format!("Failed to deserialize session: {}", e))?;
            return Ok(Some(stored.session));
        }
        Ok(None)
    }

    /// Clears the session from localStorage
    pub fn clear_session() -> Result<(), String> {
        if let Some(window) = web_sys::window()
            && let Ok(Some(storage)) = window.local_storage()
        {
            storage
                .remove_item(Self::SESSION_KEY)
                .map_err(|e| format!("Failed to clear session: {:?}", e))?;
            return Ok(());
        }
        Err("Failed to access localStorage".to_string())
    }

    /// Checks if a session exists and is valid
    pub fn has_valid_session() -> bool {
        unimplemented!("has_valid_session not implemented")
    }
}

impl Default for SessionStorage {
    fn default() -> Self {
        Self
    }
}
