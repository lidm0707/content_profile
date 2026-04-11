use tracing::{debug, info, trace, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Office,
    Supabase,
}

#[derive(Debug, Clone)]
pub struct Config {
    pub mode: AppMode,
    pub supabase_url: Option<String>,
    pub supabase_anon_key: Option<String>,
    pub sync_enabled: bool,
}

impl Config {
    pub fn wasm_config() -> Self {
        let mode = match env!("APP_MODE") {
            "supabase" => {
                info!("APP_MODE set to 'supabase'");
                AppMode::Supabase
            }
            mode_str if !mode_str.is_empty() => {
                debug!("APP_MODE set to '{}', defaulting to Office mode", mode_str);
                AppMode::Office
            }
            _ => {
                debug!("APP_MODE not set, defaulting to Office mode");
                AppMode::Office
            }
        };

        let sync_enabled = match env!("SYNC_ENABLED") {
            "true" | "1" => {
                info!("Sync enabled via SYNC_ENABLED environment variable");
                true
            }
            _ => {
                debug!("Sync disabled (SYNC_ENABLED not set to 'true' or '1')");
                false
            }
        };

        let supabase_url = env!("SUPABASE_URL");
        let supabase_anon_key = env!("SUPABASE_ANON_KEY");

        let (supabase_url_opt, supabase_anon_key_opt) = if mode == AppMode::Supabase {
            if supabase_url.is_empty() {
                warn!("SUPABASE_URL not set");
                warn!("WARNING: Running in Supabase mode but SUPABASE_URL is not configured!");
                (None, None)
            } else {
                info!(
                    "SUPABASE_URL found: {}***",
                    &supabase_url[..supabase_url.len().min(20)]
                );
                if supabase_anon_key.is_empty() {
                    warn!("SUPABASE_ANON_KEY not set");
                    warn!(
                        "WARNING: Running in Supabase mode but SUPABASE_ANON_KEY is not configured!"
                    );
                    (Some(supabase_url.to_string()), None)
                } else {
                    debug!(
                        "SUPABASE_ANON_KEY found: {}***",
                        &supabase_anon_key[..supabase_anon_key.len().min(10)]
                    );
                    (
                        Some(supabase_url.to_string()),
                        Some(supabase_anon_key.to_string()),
                    )
                }
            }
        } else {
            if supabase_url.is_empty() {
                debug!("SUPABASE_URL not set");
            } else {
                info!(
                    "SUPABASE_URL found: {}***",
                    &supabase_url[..supabase_url.len().min(20)]
                );
            }
            if supabase_anon_key.is_empty() {
                debug!("SUPABASE_ANON_KEY not set");
            } else {
                info!(
                    "SUPABASE_ANON_KEY found: {}***",
                    &supabase_anon_key[..supabase_anon_key.len().min(10)]
                );
            }
            (
                if supabase_url.is_empty() {
                    None
                } else {
                    Some(supabase_url.to_string())
                },
                if supabase_anon_key.is_empty() {
                    None
                } else {
                    Some(supabase_anon_key.to_string())
                },
            )
        };

        let config = Config {
            mode,
            supabase_url: supabase_url_opt,
            supabase_anon_key: supabase_anon_key_opt,
            sync_enabled,
        };

        info!(
            "Configuration loaded: mode={:?}, sync_enabled={}, supabase_url={:?}",
            config.mode,
            config.sync_enabled,
            config.supabase_url.is_some()
        );

        config
    }

    pub fn new() -> Result<Self, String> {
        todo!("Implement Config::new for different environments")
    }

    pub fn dev_config() -> Self {
        info!("Loading configuration...");
        Self::wasm_config()
    }

    pub fn is_office_mode(&self) -> bool {
        self.mode == AppMode::Office
    }

    pub fn is_supabase_mode(&self) -> bool {
        self.mode == AppMode::Supabase
    }

    pub fn is_sync_enabled(&self) -> bool {
        self.sync_enabled
    }

    pub fn get_supabase_url(&self) -> Option<&String> {
        todo!("Implement get_supabase_url accessor")
    }

    pub fn get_supabase_anon_key(&self) -> Option<&String> {
        todo!("Implement get_supabase_anon_key accessor")
    }
}

pub fn get_config() -> Config {
    trace!("get_config() called");
    Config::dev_config()
}
