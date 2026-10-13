use std::sync::LazyLock;
use std::sync::RwLock;

struct AppState {
    destination_path: String,
}

static APP_STATE: LazyLock<RwLock<AppState>> = LazyLock::new(|| {
    RwLock::new(
        AppState {
            destination_path: "".to_string(),
        }
    )
});

#[tauri::command]
fn set_destination_path(path: &str) {

    println!("path: {}", path);

    if let Ok(mut app_state) = APP_STATE.write() {
        app_state.destination_path = path.to_string();
    }
}

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![greet, set_destination_path])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
