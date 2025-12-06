use crate::discovery::{DiscoveryService, PeerInfo};
use crate::transfer::{
    self, FileMetadata, TransferInfo, TransferMessage, TransferStatus,
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;
use tauri::{AppHandle, Emitter};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use uuid::Uuid;

const SERVER_PORT: u16 = 42070;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub platform: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub connected: bool,
    pub peer: Option<PeerInfo>,
    pub connection_type: Option<String>,
}

pub struct AppState {
    app: AppHandle,
    device_id: String,
    device_name: String,
    discovery: Mutex<Option<DiscoveryService>>,
    connected_peer: RwLock<Option<PeerInfo>>,
    connection: Mutex<Option<TcpStream>>,
    transfers: RwLock<HashMap<String, TransferInfo>>,
    download_path: RwLock<PathBuf>,
}

impl AppState {
    pub fn new(app: AppHandle) -> Self {
        let device_id = get_or_create_device_id();
        let device_name = get_device_name();
        let download_path = get_default_download_path();
        
        Self {
            app,
            device_id,
            device_name,
            discovery: Mutex::new(None),
            connected_peer: RwLock::new(None),
            connection: Mutex::new(None),
            transfers: RwLock::new(HashMap::new()),
            download_path: RwLock::new(download_path),
        }
    }
    
    pub async fn get_device_info(&self) -> DeviceInfo {
        DeviceInfo {
            id: self.device_id.clone(),
            name: self.device_name.clone(),
            platform: get_platform(),
        }
    }
    
    pub async fn start_discovery(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut discovery_guard = self.discovery.lock().await;
        
        if discovery_guard.is_some() {
            return Ok(()); // Already running
        }
        
        let mut discovery = DiscoveryService::new(
            self.device_id.clone(),
            self.device_name.clone(),
            SERVER_PORT,
        );
        
        discovery.start()?;
        *discovery_guard = Some(discovery);
        
        // Start emitting peer updates
        let app = self.app.clone();
        let discovery_ref = self.discovery.lock().await;
        if let Some(ref disc) = *discovery_ref {
            let peers_arc = Arc::new(RwLock::new(disc.get_peers()));
            drop(discovery_ref);
            
            let app_clone = app.clone();
            tokio::spawn(async move {
                loop {
                    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
                    // Emit peer update event
                    let _ = app_clone.emit("peers-updated", ());
                }
            });
        }
        
        Ok(())
    }
    
    pub async fn stop_discovery(&self) {
        let mut discovery_guard = self.discovery.lock().await;
        if let Some(ref mut discovery) = *discovery_guard {
            discovery.stop();
        }
        *discovery_guard = None;
    }
    
    pub async fn get_peers(&self) -> Vec<PeerInfo> {
        let discovery_guard = self.discovery.lock().await;
        if let Some(ref discovery) = *discovery_guard {
            discovery.get_peers()
        } else {
            Vec::new()
        }
    }
    
    pub async fn connect_to_peer(&self, peer_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let peers = self.get_peers().await;
        let peer = peers.iter().find(|p| p.id == peer_id).cloned()
            .ok_or("Peer not found")?;
        
        let addr = format!("{}:{}", peer.ip, peer.port);
        let stream = TcpStream::connect(&addr).await?;
        
        *self.connected_peer.write() = Some(peer.clone());
        *self.connection.lock().await = Some(stream);
        
        // Emit connection event
        let _ = self.app.emit("peer-connected", &peer);
        
        Ok(())
    }
    
    pub async fn disconnect_peer(&self) {
        *self.connected_peer.write() = None;
        *self.connection.lock().await = None;
        let _ = self.app.emit("peer-disconnected", ());
    }
    
    pub async fn get_connection_status(&self) -> ConnectionStatus {
        let peer = self.connected_peer.read().clone();
        let connected = peer.is_some();
        
        ConnectionStatus {
            connected,
            peer,
            connection_type: if connected { Some("TCP".to_string()) } else { None },
        }
    }
    
    pub async fn start_server(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let listener = TcpListener::bind(format!("0.0.0.0:{}", SERVER_PORT)).await?;
        println!("Thunderlink server listening on port {}", SERVER_PORT);
        
        let app = self.app.clone();
        let download_path = self.download_path.read().clone();
        
        loop {
            let (stream, addr) = listener.accept().await?;
            println!("Incoming connection from {}", addr);
            
            let app_clone = app.clone();
            let download_path_clone = download_path.clone();
            
            tokio::spawn(async move {
                if let Err(e) = handle_incoming_connection(stream, app_clone, download_path_clone).await {
                    eprintln!("Connection error: {}", e);
                }
            });
        }
    }
    
    pub async fn send_files(
        &self,
        file_paths: Vec<String>,
        app: AppHandle,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let mut connection_guard = self.connection.lock().await;
        let stream = connection_guard.as_mut()
            .ok_or("Not connected to a peer")?;
        
        let transfer_id = Uuid::new_v4().to_string();
        
        // Prepare file metadata
        let mut files_metadata = Vec::new();
        for path_str in &file_paths {
            let path = PathBuf::from(path_str);
            let name = path.file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "unknown".to_string());
            let metadata = tokio::fs::metadata(&path).await?;
            let checksum = transfer::calculate_checksum(&path).await?;
            
            files_metadata.push(FileMetadata {
                name,
                size: metadata.len(),
                checksum,
            });
        }
        
        // Create transfer info for each file
        for (i, meta) in files_metadata.iter().enumerate() {
            let info = TransferInfo::new(
                format!("{}_{}", transfer_id, i),
                meta.name.clone(),
                meta.size,
                true,
            );
            self.transfers.write().insert(info.id.clone(), info);
        }
        
        // Send file offer
        let offer = TransferMessage::FileOffer {
            transfer_id: transfer_id.clone(),
            files: files_metadata.clone(),
        };
        transfer::send_message(stream, &offer).await?;
        
        // Wait for acceptance
        let response = transfer::receive_message(stream).await?;
        match response {
            TransferMessage::AcceptTransfer { .. } => {
                // Start sending files
                for (i, path_str) in file_paths.iter().enumerate() {
                    let path = PathBuf::from(path_str);
                    let info_id = format!("{}_{}", transfer_id, i);
                    
                    // Update status
                    if let Some(info) = self.transfers.write().get_mut(&info_id) {
                        info.status = TransferStatus::Transferring;
                    }
                    let _ = app.emit("transfer-updated", &info_id);
                    
                    let app_clone = app.clone();
                    let info_id_clone = info_id.clone();
                    
                    transfer::stream_file(stream, &path, &transfer_id, i, |bytes| {
                        let _ = app_clone.emit("transfer-progress", (&info_id_clone, bytes));
                    }).await?;
                    
                    // Update transferred bytes after completion
                    let file_size = files_metadata[i].size;
                    if let Some(info) = self.transfers.write().get_mut(&info_id) {
                        info.transferred = file_size;
                    }
                    
                    // Mark as completed
                    if let Some(info) = self.transfers.write().get_mut(&info_id) {
                        info.status = TransferStatus::Completed;
                    }
                    let _ = app.emit("transfer-updated", &info_id);
                }
            }
            TransferMessage::RejectTransfer { reason, .. } => {
                return Err(format!("Transfer rejected: {}", reason).into());
            }
            _ => {
                return Err("Unexpected response".into());
            }
        }
        
        Ok(transfer_id)
    }
    
    pub async fn get_transfers(&self) -> Vec<TransferInfo> {
        self.transfers.read().values().cloned().collect()
    }
    
    pub async fn cancel_transfer(&self, transfer_id: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        if let Some(info) = self.transfers.write().get_mut(transfer_id) {
            info.status = TransferStatus::Cancelled;
        }
        Ok(())
    }
    
    pub async fn set_download_path(&self, path: String) {
        *self.download_path.write() = PathBuf::from(path);
    }
    
    pub async fn get_download_path(&self) -> String {
        self.download_path.read().to_string_lossy().to_string()
    }
}

async fn handle_incoming_connection(
    mut stream: TcpStream,
    app: AppHandle,
    download_path: PathBuf,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    loop {
        let msg = match transfer::receive_message(&mut stream).await {
            Ok(m) => m,
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => break,
            Err(e) => return Err(e.into()),
        };
        
        match msg {
            TransferMessage::FileOffer { transfer_id, files } => {
                // Emit event to UI for user to accept/reject
                let _ = app.emit("incoming-transfer", (&transfer_id, &files));
                
                // Auto-accept for now (could be made interactive)
                let accept = TransferMessage::AcceptTransfer {
                    transfer_id: transfer_id.clone(),
                };
                transfer::send_message(&mut stream, &accept).await?;
                
                // Receive files
                for (i, file_meta) in files.iter().enumerate() {
                    let save_path = download_path.join(&file_meta.name);
                    
                    let _ = app.emit("receiving-file", (&transfer_id, i, &file_meta.name));
                    
                    let app_clone = app.clone();
                    let transfer_id_clone = transfer_id.clone();
                    
                    let verified = transfer::receive_file(
                        &mut stream,
                        &save_path,
                        file_meta.size,
                        &file_meta.checksum,
                        |bytes| {
                            let _ = app_clone.emit("receive-progress", (&transfer_id_clone, i, bytes));
                        },
                    ).await?;
                    
                    let _ = app.emit("file-received", (&transfer_id, i, &file_meta.name, verified));
                }
            }
            TransferMessage::TransferComplete { transfer_id } => {
                let _ = app.emit("transfer-complete", &transfer_id);
            }
            _ => {}
        }
    }
    
    Ok(())
}

fn get_or_create_device_id() -> String {
    // Try to load existing ID or create new one
    if let Some(proj_dirs) = directories::ProjectDirs::from("com", "thunderlink", "Thunderlink") {
        let config_path = proj_dirs.config_dir().join("device_id");
        
        if let Ok(id) = std::fs::read_to_string(&config_path) {
            return id.trim().to_string();
        }
        
        let new_id = Uuid::new_v4().to_string();
        let _ = std::fs::create_dir_all(proj_dirs.config_dir());
        let _ = std::fs::write(&config_path, &new_id);
        return new_id;
    }
    
    Uuid::new_v4().to_string()
}

fn get_device_name() -> String {
    hostname::get()
        .map(|h| h.to_string_lossy().to_string())
        .unwrap_or_else(|_| "Unknown Device".to_string())
}

fn get_default_download_path() -> PathBuf {
    directories::UserDirs::new()
        .and_then(|dirs| dirs.download_dir().map(|p| p.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn get_platform() -> String {
    #[cfg(target_os = "macos")]
    return "macOS".to_string();
    
    #[cfg(target_os = "windows")]
    return "Windows".to_string();
    
    #[cfg(target_os = "linux")]
    return "Linux".to_string();
    
    #[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
    return "Unknown".to_string();
}

