use std::env;

// Learn more about Tauri commands at https://v2.tauri.app/develop/calling-rust/#commands
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Try to load .env file if it exists
    if let Err(_) = dotenvy::dotenv() {
        println!("No .env file found, using environment variables or defaults");
    }
    
    let api_key = env::var("POSTHOG_API_KEY")
        .expect("POSTHOG_API_KEY environment variable is required. Please set it in .env file or environment.");
    
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .plugin(tauri_plugin_posthog::init(tauri_plugin_posthog::PostHogConfig {
            api_key,
            ..Default::default()
        }))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
