mod inbox;
mod launcher;
mod switchboard;

use std::sync::{Arc, Mutex};

#[tauri::command]
fn current_sessions(board: tauri::State<inbox::SharedBoard>) -> Vec<switchboard::SessionView> {
    board.lock().unwrap().sessions()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let board: inbox::SharedBoard = Arc::new(Mutex::new(switchboard::Board::default()));
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(board.clone())
        .setup(move |app| {
            inbox::start(app.handle().clone(), board);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![current_sessions, launcher::launch_session, launcher::default_cwd])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
