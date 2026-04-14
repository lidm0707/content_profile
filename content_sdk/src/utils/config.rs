#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppMode {
    Office,
    Supabase,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub mode: AppMode,
    pub supabase_url: Option<String>,
    pub supabase_anon_key: Option<String>,
    pub jwt_token: Option<String>,
}

impl Config {
    pub fn new(
        mode: &str,
        supabase_url: &str,
        supabase_anon_key: &str,
        jwt_token: Option<String>,
    ) -> Self {
        let mode = match mode {
            "supabase" => AppMode::Supabase,
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

        Config {
            mode,
            supabase_url: supabase_url_opt,
            supabase_anon_key: supabase_anon_key_opt,
            jwt_token,
        }
    }

    pub fn is_office_mode(&self) -> bool {
        self.mode == AppMode::Office
    }

    pub fn is_supabase_mode(&self) -> bool {
        self.mode == AppMode::Supabase
    }
}
