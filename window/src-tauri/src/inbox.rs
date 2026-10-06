//! 兩條背景執行緒：HTTP 收 mod 的狀態回報；每 5 秒掃一次 session 檔，移除已經不在的 session。
//! 每次 session 表有變化，就把排好序的清單用 `sessions-changed` 事件推給前端。
use crate::switchboard::{Board, Report, SessionInfo};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::{AppHandle, Emitter};

pub const ADDRESS: &str = "127.0.0.1:47321";
const SCAN_INTERVAL: Duration = Duration::from_secs(5);

pub type SharedBoard = Arc<Mutex<Board>>;

fn publish(app: &AppHandle, board: &SharedBoard) {
    let sessions = board.lock().unwrap().sessions();
    let _ = app.emit("sessions-changed", sessions);
}

pub fn start(app: AppHandle, board: SharedBoard) {
    let http_app = app.clone();
    let http_board = board.clone();
    match tiny_http::Server::http(ADDRESS) {
        Ok(server) => {
            std::thread::spawn(move || serve(server, http_app, http_board));
        }
        Err(e) => eprintln!("tsunu-pet: {ADDRESS} 開不起來（可能有另一個總機在跑）：{e}"),
    }

    std::thread::spawn(move || loop {
        let live = read_session_files();
        board.lock().unwrap().retain_live(&live);
        publish(&app, &board);
        std::thread::sleep(SCAN_INTERVAL);
    });
}

fn serve(server: tiny_http::Server, app: AppHandle, board: SharedBoard) {
    for mut request in server.incoming_requests() {
        let mut body = String::new();
        let parsed = request
            .as_reader()
            .read_to_string(&mut body)
            .ok()
            .and_then(|_| serde_json::from_str::<Report>(&body).ok());
        let status = match parsed {
            Some(report) => {
                board.lock().unwrap().apply(report);
                publish(&app, &board);
                204
            }
            None => 400,
        };
        let _ = request.respond(tiny_http::Response::empty(status));
    }
}

fn read_session_files() -> Vec<SessionInfo> {
    let dir = crate::launcher::home_dir().join(".claude").join("sessions");
    let Ok(entries) = std::fs::read_dir(dir) else { return Vec::new() };
    entries
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext == "json"))
        .filter_map(|entry| std::fs::read_to_string(entry.path()).ok())
        .filter_map(|text| serde_json::from_str::<serde_json::Value>(&text).ok())
        .filter_map(|info| {
            let session_id = info["sessionId"].as_str()?.to_string();
            let name = match info["name"].as_str() {
                Some(name) => name.to_string(),
                None => session_id.chars().take(8).collect(),
            };
            Some(SessionInfo {
                name,
                background: info["kind"].as_str() == Some("bg"),
                session_id,
            })
        })
        .collect()
}
