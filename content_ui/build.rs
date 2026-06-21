fn main() {
    dotenvy::dotenv().ok();

    println!("cargo:rerun-if-changed=.env");

    // Provide default values for missing environment variables
    let app_mode = std::env::var("APP_MODE").expect("not found ENV: APP_MODE");
    let supabase_url = std::env::var("SUPABASE_URL").expect("not found ENV: SUPABASE_URL");
    let supabase_anon_key =
        std::env::var("SUPABASE_ANON_KEY").expect("not found ENV: SUPABASE_ANON_KEY");
    let google_oauth_client_id = std::env::var("GOOGLE_OAUTH_CLIENT_ID").unwrap_or_default();
    let google_drive_folder_id = std::env::var("GOOGLE_DRIVE_FOLDER_ID").unwrap_or_default();

    // Print all relevant env vars (including defaults)
    println!("cargo:rustc-env=APP_MODE={}", app_mode);
    println!("cargo:rustc-env=SUPABASE_URL={}", supabase_url);
    println!("cargo:rustc-env=SUPABASE_ANON_KEY={}", supabase_anon_key);
    println!(
        "cargo:rustc-env=GOOGLE_OAUTH_CLIENT_ID={}",
        google_oauth_client_id
    );
    println!(
        "cargo:rustc-env=GOOGLE_DRIVE_FOLDER_ID={}",
        google_drive_folder_id
    );

    // Also print any other SUPABASE_ prefixed variables
    for (key, value) in std::env::vars() {
        if key.starts_with("SUPABASE_") && key != "SUPABASE_URL" && key != "SUPABASE_ANON_KEY" {
            println!("cargo:rustc-env={}={}", key, value);
        }
    }
}
