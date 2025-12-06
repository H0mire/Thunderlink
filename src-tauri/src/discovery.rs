use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::{SocketAddr, UdpSocket};
use std::sync::Arc;
use std::time::{Duration, Instant};
use parking_lot::RwLock;
use tokio::sync::broadcast;

const DISCOVERY_PORT: u16 = 42069;
const BEACON_INTERVAL: Duration = Duration::from_secs(1);
const PEER_TIMEOUT: Duration = Duration::from_secs(5);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: String,
    pub name: String,
    pub ip: String,
    pub port: u16,
    pub platform: String,
    #[serde(skip)]
    pub last_seen: Option<Instant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct DiscoveryMessage {
    pub id: String,
    pub name: String,
    pub port: u16,
    pub platform: String,
}

pub struct DiscoveryService {
    device_id: String,
    device_name: String,
    server_port: u16,
    peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
    stop_tx: Option<broadcast::Sender<()>>,
}

impl DiscoveryService {
    pub fn new(device_id: String, device_name: String, server_port: u16) -> Self {
        Self {
            device_id,
            device_name,
            server_port,
            peers: Arc::new(RwLock::new(HashMap::new())),
            stop_tx: None,
        }
    }
    
    pub fn start(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let (stop_tx, _) = broadcast::channel(1);
        self.stop_tx = Some(stop_tx.clone());
        
        // Start beacon sender
        let device_id = self.device_id.clone();
        let device_name = self.device_name.clone();
        let server_port = self.server_port;
        let mut stop_rx = stop_tx.subscribe();
        
        tokio::spawn(async move {
            let msg = DiscoveryMessage {
                id: device_id,
                name: device_name,
                port: server_port,
                platform: get_platform(),
            };
            let msg_bytes = serde_json::to_vec(&msg).unwrap();
            
            loop {
                tokio::select! {
                    _ = stop_rx.recv() => break,
                    _ = tokio::time::sleep(BEACON_INTERVAL) => {
                        if let Err(e) = send_beacon(&msg_bytes) {
                            eprintln!("Failed to send beacon: {}", e);
                        }
                    }
                }
            }
        });
        
        // Start beacon listener
        let peers = self.peers.clone();
        let my_id = self.device_id.clone();
        let mut stop_rx = stop_tx.subscribe();
        
        tokio::spawn(async move {
            let socket = match UdpSocket::bind(format!("0.0.0.0:{}", DISCOVERY_PORT)) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Failed to bind discovery socket: {}", e);
                    return;
                }
            };
            socket.set_nonblocking(true).ok();
            
            let mut buf = [0u8; 1024];
            
            loop {
                tokio::select! {
                    _ = stop_rx.recv() => break,
                    _ = tokio::time::sleep(Duration::from_millis(100)) => {
                        // Try to receive beacons
                        while let Ok((len, addr)) = socket.recv_from(&mut buf) {
                            if let Ok(msg) = serde_json::from_slice::<DiscoveryMessage>(&buf[..len]) {
                                // Ignore our own beacons
                                if msg.id == my_id {
                                    continue;
                                }
                                
                                let peer = PeerInfo {
                                    id: msg.id.clone(),
                                    name: msg.name,
                                    ip: addr.ip().to_string(),
                                    port: msg.port,
                                    platform: msg.platform,
                                    last_seen: Some(Instant::now()),
                                };
                                
                                peers.write().insert(msg.id, peer);
                            }
                        }
                        
                        // Clean up stale peers
                        peers.write().retain(|_, peer| {
                            peer.last_seen
                                .map(|t| t.elapsed() < PEER_TIMEOUT)
                                .unwrap_or(false)
                        });
                    }
                }
            }
        });
        
        Ok(())
    }
    
    pub fn stop(&mut self) {
        if let Some(tx) = self.stop_tx.take() {
            let _ = tx.send(());
        }
    }
    
    pub fn get_peers(&self) -> Vec<PeerInfo> {
        self.peers.read().values().cloned().collect()
    }
}

fn send_beacon(msg: &[u8]) -> Result<(), std::io::Error> {
    let socket = UdpSocket::bind("0.0.0.0:0")?;
    socket.set_broadcast(true)?;
    
    // Send to broadcast on all interfaces
    let broadcast_addrs = vec![
        format!("255.255.255.255:{}", DISCOVERY_PORT),
        format!("224.0.0.1:{}", DISCOVERY_PORT), // Multicast
    ];
    
    for addr in broadcast_addrs {
        if let Ok(addr) = addr.parse::<SocketAddr>() {
            let _ = socket.send_to(msg, addr);
        }
    }
    
    Ok(())
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

