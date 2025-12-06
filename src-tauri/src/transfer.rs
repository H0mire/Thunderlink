use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use std::path::Path;
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader, BufWriter};
use tokio::net::TcpStream;

const CHUNK_SIZE: usize = 1024 * 1024; // 1MB chunks for high-speed transfer

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TransferStatus {
    Pending,
    Connecting,
    Transferring,
    Verifying,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferInfo {
    pub id: String,
    pub file_name: String,
    pub file_size: u64,
    pub transferred: u64,
    pub speed_bps: u64,
    pub status: TransferStatus,
    pub is_sending: bool,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetadata {
    pub name: String,
    pub size: u64,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransferMessage {
    // Sender -> Receiver
    FileOffer {
        transfer_id: String,
        files: Vec<FileMetadata>,
    },
    FileData {
        transfer_id: String,
        file_index: usize,
        chunk_index: u64,
        data: Vec<u8>,
        is_last: bool,
    },
    TransferComplete {
        transfer_id: String,
    },
    
    // Receiver -> Sender
    AcceptTransfer {
        transfer_id: String,
    },
    RejectTransfer {
        transfer_id: String,
        reason: String,
    },
    ChunkReceived {
        transfer_id: String,
        file_index: usize,
        chunk_index: u64,
    },
    VerificationResult {
        transfer_id: String,
        success: bool,
    },
}

impl TransferInfo {
    pub fn new(id: String, file_name: String, file_size: u64, is_sending: bool) -> Self {
        Self {
            id,
            file_name,
            file_size,
            transferred: 0,
            speed_bps: 0,
            status: TransferStatus::Pending,
            is_sending,
            error: None,
        }
    }
    
    pub fn progress_percent(&self) -> f64 {
        if self.file_size == 0 {
            return 100.0;
        }
        (self.transferred as f64 / self.file_size as f64) * 100.0
    }
}

/// Calculate SHA256 checksum of a file
pub async fn calculate_checksum(path: &Path) -> Result<String, std::io::Error> {
    let file = File::open(path).await?;
    let mut reader = BufReader::with_capacity(CHUNK_SIZE, file);
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; CHUNK_SIZE];
    
    loop {
        let bytes_read = reader.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }
    
    Ok(hex::encode(hasher.finalize()))
}

/// Send a message over the TCP stream
pub async fn send_message(stream: &mut TcpStream, msg: &TransferMessage) -> Result<(), std::io::Error> {
    let json = serde_json::to_vec(msg).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
    let len = json.len() as u32;
    
    stream.write_all(&len.to_be_bytes()).await?;
    stream.write_all(&json).await?;
    stream.flush().await?;
    
    Ok(())
}

/// Receive a message from the TCP stream
pub async fn receive_message(stream: &mut TcpStream) -> Result<TransferMessage, std::io::Error> {
    let mut len_bytes = [0u8; 4];
    stream.read_exact(&mut len_bytes).await?;
    let len = u32::from_be_bytes(len_bytes) as usize;
    
    let mut buffer = vec![0u8; len];
    stream.read_exact(&mut buffer).await?;
    
    serde_json::from_slice(&buffer).map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))
}

/// Stream a file over TCP in chunks
pub async fn stream_file(
    stream: &mut TcpStream,
    path: &Path,
    transfer_id: &str,
    file_index: usize,
    progress_callback: impl Fn(u64),
) -> Result<(), std::io::Error> {
    let file = File::open(path).await?;
    let file_size = file.metadata().await?.len();
    let mut reader = BufReader::with_capacity(CHUNK_SIZE, file);
    let mut buffer = vec![0u8; CHUNK_SIZE];
    let mut chunk_index = 0u64;
    let mut total_sent = 0u64;
    
    loop {
        let bytes_read = reader.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        
        let is_last = total_sent + bytes_read as u64 >= file_size;
        
        let msg = TransferMessage::FileData {
            transfer_id: transfer_id.to_string(),
            file_index,
            chunk_index,
            data: buffer[..bytes_read].to_vec(),
            is_last,
        };
        
        send_message(stream, &msg).await?;
        
        total_sent += bytes_read as u64;
        chunk_index += 1;
        progress_callback(total_sent);
    }
    
    Ok(())
}

/// Receive a file from TCP stream
pub async fn receive_file(
    stream: &mut TcpStream,
    save_path: &Path,
    expected_size: u64,
    expected_checksum: &str,
    progress_callback: impl Fn(u64),
) -> Result<bool, std::io::Error> {
    let file = File::create(save_path).await?;
    let mut writer = BufWriter::with_capacity(CHUNK_SIZE, file);
    let mut total_received = 0u64;
    
    loop {
        let msg = receive_message(stream).await?;
        
        match msg {
            TransferMessage::FileData { data, is_last, .. } => {
                writer.write_all(&data).await?;
                total_received += data.len() as u64;
                progress_callback(total_received);
                
                if is_last {
                    break;
                }
            }
            _ => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Unexpected message during file transfer",
                ));
            }
        }
    }
    
    writer.flush().await?;
    drop(writer);
    
    // Verify checksum
    let actual_checksum = calculate_checksum(save_path).await?;
    Ok(actual_checksum == expected_checksum)
}

/// Format bytes per second as human-readable speed
pub fn format_speed(bps: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bps >= GB {
        format!("{:.2} GB/s", bps as f64 / GB as f64)
    } else if bps >= MB {
        format!("{:.2} MB/s", bps as f64 / MB as f64)
    } else if bps >= KB {
        format!("{:.2} KB/s", bps as f64 / KB as f64)
    } else {
        format!("{} B/s", bps)
    }
}

/// Format file size as human-readable
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    
    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} B", bytes)
    }
}

