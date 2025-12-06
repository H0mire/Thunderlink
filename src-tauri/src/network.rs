use serde::{Deserialize, Serialize};
use std::net::IpAddr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInterface {
    pub name: String,
    pub ip: String,
    pub is_thunderbolt: bool,
    pub is_usb: bool,
    pub speed_mbps: Option<u64>,
}

/// Get all network interfaces, prioritizing Thunderbolt and USB-C connections
pub fn get_interfaces() -> Result<Vec<NetworkInterface>, Box<dyn std::error::Error + Send + Sync>> {
    let mut interfaces = Vec::new();
    
    for iface in if_addrs::get_if_addrs()? {
        // Skip loopback
        if iface.is_loopback() {
            continue;
        }
        
        let ip = match iface.ip() {
            IpAddr::V4(v4) => v4.to_string(),
            IpAddr::V6(_) => continue, // Skip IPv6 for simplicity
        };
        
        let name = iface.name.clone();
        
        // Detect Thunderbolt/USB interfaces by name patterns
        let is_thunderbolt = is_thunderbolt_interface(&name);
        let is_usb = is_usb_interface(&name);
        
        interfaces.push(NetworkInterface {
            name,
            ip,
            is_thunderbolt,
            is_usb,
            speed_mbps: None, // Would need platform-specific APIs
        });
    }
    
    // Sort: Thunderbolt first, then USB, then others
    interfaces.sort_by(|a, b| {
        match (a.is_thunderbolt, b.is_thunderbolt) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => match (a.is_usb, b.is_usb) {
                (true, false) => std::cmp::Ordering::Less,
                (false, true) => std::cmp::Ordering::Greater,
                _ => a.name.cmp(&b.name),
            }
        }
    });
    
    Ok(interfaces)
}

/// Detect if interface is a Thunderbolt bridge
fn is_thunderbolt_interface(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    
    // macOS: Thunderbolt Bridge creates "bridge" interfaces
    if name_lower.contains("bridge") {
        return true;
    }
    
    // Linux: Thunderbolt networking shows as thunderbolt interfaces
    if name_lower.contains("thunderbolt") || name_lower.contains("tb") {
        return true;
    }
    
    // Windows: Thunderbolt appears as special Ethernet adapters
    if name_lower.contains("thunderbolt") {
        return true;
    }
    
    false
}

/// Detect if interface is a USB-C/USB network adapter
fn is_usb_interface(name: &str) -> bool {
    let name_lower = name.to_lowercase();
    
    // Common USB network interface patterns
    name_lower.contains("usb") 
        || name_lower.contains("rndis")  // Windows USB tethering
        || name_lower.contains("cdc")     // USB CDC Ethernet
        || name_lower.contains("ncm")     // USB NCM
}

/// Get the best available IP for peer-to-peer transfer
pub fn get_best_transfer_ip() -> Option<String> {
    let interfaces = get_interfaces().ok()?;
    
    // First try Thunderbolt
    if let Some(tb) = interfaces.iter().find(|i| i.is_thunderbolt) {
        return Some(tb.ip.clone());
    }
    
    // Then USB
    if let Some(usb) = interfaces.iter().find(|i| i.is_usb) {
        return Some(usb.ip.clone());
    }
    
    // Fall back to any non-loopback interface
    interfaces.first().map(|i| i.ip.clone())
}

/// Get all local IPs for broadcasting presence
pub fn get_all_local_ips() -> Vec<String> {
    get_interfaces()
        .unwrap_or_default()
        .into_iter()
        .map(|i| i.ip)
        .collect()
}

