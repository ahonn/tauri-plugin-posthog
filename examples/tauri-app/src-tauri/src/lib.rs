// Learn more about Tauri commands at https://v2.tauri.app/develop/calling-rust/#commands
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet])
        .plugin(tauri_plugin_posthog::init(tauri_plugin_posthog::PostHogConfig {
            api_key: "phc_test_key_please_replace_with_your_own".to_string(),
            api_endpoint: "https://us.i.posthog.com".to_string(),
            request_timeout_seconds: 30,
            auto_capture: true,  // Enable auto-identify with device-based distinct_id
        }))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
