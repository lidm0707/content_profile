#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum AppMode {
    #[default]
    Office,
    Supabase,
    Backend,
}

impl AppMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            AppMode::Office => "office",
            AppMode::Supabase => "supabase",
            AppMode::Backend => "backend",
        }
    }
}

impl std::str::FromStr for AppMode {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "office" => Ok(AppMode::Office),
            "supabase" => Ok(AppMode::Supabase),
            "backend" => Ok(AppMode::Backend),
            other => Err(format!("unknown app mode: {other}")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub mode: AppMode,
    pub supabase_url: Option<String>,
    pub supabase_anon_key: Option<String>,
    pub jwt_token: Option<String>,
    pub google_oauth_client_id: Option<String>,
    /// Base URL of the content_backend HTTP API (backend mode).
    /// Empty means same-origin (`/api/...`).
    pub backend_api_url: Option<String>,
}

impl Config {
    pub fn new(
        mode: &str,
        supabase_url: &str,
        supabase_anon_key: &str,
        jwt_token: Option<String>,
        google_oauth_client_id: Option<String>,
    ) -> Self {
        let mode = match mode {
            "supabase" => AppMode::Supabase,
            "backend" => AppMode::Backend,
            _ => AppMode::Office,
        };

        let (supabase_url_opt, supabase_anon_key_opt) = if mode == AppMode::Supabase {
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
        } else {
            (None, None)
        };

        let google_oauth_client_id_opt = if google_oauth_client_id
            .as_deref()
            .unwrap_or_default()
            .is_empty()
        {
            None
        } else {
            google_oauth_client_id
        };

        Config {
            mode,
            supabase_url: supabase_url_opt,
            supabase_anon_key: supabase_anon_key_opt,
            jwt_token,
            google_oauth_client_id: google_oauth_client_id_opt,
            backend_api_url: None,
        }
    }

    /// Sets the content_backend API base URL (backend mode).
    pub fn with_backend_api_url(mut self, url: Option<String>) -> Self {
        self.backend_api_url = url.filter(|u| !u.is_empty());
        self
    }

    /// Base URL for backend API calls: configured value or same-origin.
    pub fn api_base(&self) -> String {
        self.backend_api_url.clone().unwrap_or_default()
    }

    pub fn is_office_mode(&self) -> bool {
        self.mode == AppMode::Office
    }

    pub fn is_supabase_mode(&self) -> bool {
        self.mode == AppMode::Supabase
    }
}
