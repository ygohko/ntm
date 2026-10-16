use std::sync::LazyLock;
use std::sync::RwLock;

use ntm_model::backup_executer::BackupExecuter;
use ntm_model::task::Task;

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

#[tauri::command]
fn execute() -> bool {

    println!("Executing backup...");

    let app_state = {
        let Ok(app_state) = APP_STATE.read() else {
            return false;
        };

        app_state
    };

    let mut executer = BackupExecuter::new(&app_state.destination_path);
    let result = match executer.execute() {
        Ok(_) => {
            println!("Execution succeeded.");

            true
        },
        Err(error) => {
            println!("Execution failed. error: {}", error);

            false
        }
    };

    result
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![set_destination_path, execute])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
