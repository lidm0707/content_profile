fn main() {
    dotenvy::dotenv().ok();

    println!("cargo:rerun-if-changed=.env");

    // Provide default values for missing environment variables
    let sync_enabled = std::env::var("SYNC_ENABLED").unwrap_or_else(|_| "false".to_string());
    let app_mode = std::env::var("APP_MODE").unwrap_or_else(|_| "office".to_string());
    let supabase_url = std::env::var("SUPABASE_URL").unwrap_or_else(|_| "".to_string());
    let supabase_anon_key = std::env::var("SUPABASE_ANON_KEY").unwrap_or_else(|_| "".to_string());

    // Print all relevant env vars (including defaults)
    println!("cargo:rustc-env=SYNC_ENABLED={}", sync_enabled);
    println!("cargo:rustc-env=APP_MODE={}", app_mode);
    println!("cargo:rustc-env=SUPABASE_URL={}", supabase_url);
    println!("cargo:rustc-env=SUPABASE_ANON_KEY={}", supabase_anon_key);

    // Also print any other SUPABASE_ prefixed variables
    for (key, value) in std::env::vars() {
        if key.starts_with("SUPABASE_") && key != "SUPABASE_URL" && key != "SUPABASE_ANON_KEY" {
            println!("cargo:rustc-env={}={}", key, value);
        }
    }
}
