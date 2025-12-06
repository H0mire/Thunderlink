mod network;
mod transfer;
mod discovery;
mod state;

use state::AppState;
use tauri::Manager;
use std::sync::Arc;

#[tauri::command]
async fn get_device_info(state: tauri::State<'_, Arc<AppState>>) -> Result<serde_json::Value, String> {
    let info = state.get_device_info().await;
    Ok(serde_json::to_value(info).map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn start_discovery(state: tauri::State<'_, Arc<AppState>>) -> Result<(), String> {
    state.start_discovery().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn stop_discovery(state: tauri::State<'_, Arc<AppState>>) -> Result<(), String> {
    state.stop_discovery().await;
    Ok(())
}

#[tauri::command]
async fn get_peers(state: tauri::State<'_, Arc<AppState>>) -> Result<Vec<discovery::PeerInfo>, String> {
    Ok(state.get_peers().await)
}

#[tauri::command]
async fn connect_to_peer(
    peer_id: String,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    state.connect_to_peer(&peer_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn disconnect_peer(state: tauri::State<'_, Arc<AppState>>) -> Result<(), String> {
    state.disconnect_peer().await;
    Ok(())
}

#[tauri::command]
async fn get_connection_status(state: tauri::State<'_, Arc<AppState>>) -> Result<serde_json::Value, String> {
    let status = state.get_connection_status().await;
    Ok(serde_json::to_value(status).map_err(|e| e.to_string())?)
}

#[tauri::command]
async fn send_files(
    file_paths: Vec<String>,
    state: tauri::State<'_, Arc<AppState>>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    state.send_files(file_paths, app).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_transfers(state: tauri::State<'_, Arc<AppState>>) -> Result<Vec<transfer::TransferInfo>, String> {
    Ok(state.get_transfers().await)
}

#[tauri::command]
async fn cancel_transfer(
    transfer_id: String,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    state.cancel_transfer(&transfer_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_network_interfaces() -> Result<Vec<network::NetworkInterface>, String> {
    network::get_interfaces().map_err(|e| e.to_string())
}

#[tauri::command]
async fn set_download_path(
    path: String,
    state: tauri::State<'_, Arc<AppState>>,
) -> Result<(), String> {
    state.set_download_path(path).await;
    Ok(())
}

#[tauri::command]
async fn get_download_path(state: tauri::State<'_, Arc<AppState>>) -> Result<String, String> {
    Ok(state.get_download_path().await)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_state = Arc::new(AppState::new(app.handle().clone()));
            app.manage(app_state.clone());
            
            // Start the TCP server for incoming connections
            let state_clone = app_state.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(e) = state_clone.start_server().await {
                    eprintln!("Failed to start server: {}", e);
                }
            });
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_device_info,
            start_discovery,
            stop_discovery,
            get_peers,
            connect_to_peer,
            disconnect_peer,
            get_connection_status,
            send_files,
            get_transfers,
            cancel_transfer,
            get_network_interfaces,
            set_download_path,
            get_download_path,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

