//! Leuwi Panjang Remote Access Plugin
//!
//! Enables remote connection to the terminal from mobile/laptop
//! via encrypted WireGuard tunnel. No public IP needed.
//!
//! Flow:
//! 1. Desktop starts embedded server (this plugin)
//! 2. Server generates keypair + pairing code
//! 3. Desktop shows QR code / 6-digit code
//! 4. Mobile scans QR → auto-configures tunnel
//! 5. Mobile connects via WireGuard → accesses terminal + AI
//!
//! Features:
//! - Zero-config WireGuard tunnel (no manual key/config management)
//! - QR code pairing (scan from mobile)
//! - 6-digit code pairing (manual entry fallback)
//! - Device management (list, revoke)
//! - Per-device permissions (terminal view/write, AI access, file transfer)
//! - Audit trail of all remote actions

use serde::{Deserialize, Serialize};

/// Server state
pub struct RemoteServer {
    pub private_key: [u8; 32],
    pub public_key: [u8; 32],
    pub listen_port: u16,
    pub devices: Vec<PairedDevice>,
    pub running: bool,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct PairedDevice {
    pub name: String,
    pub public_key: [u8; 32],
    pub internal_ip: String,
    pub permissions: DevicePermissions,
    pub paired_at: String,
    pub last_seen: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct DevicePermissions {
    pub terminal_view: bool,
    pub terminal_write: bool,
    pub ai_access: bool,
    pub file_transfer: bool,
    pub ssh_proxy: bool,
}

impl Default for DevicePermissions {
    fn default() -> Self {
        Self {
            terminal_view: true,
            terminal_write: true,
            ai_access: true,
            file_transfer: false,
            ssh_proxy: false,
        }
    }
}

/// Pairing info encoded in QR code
#[derive(Serialize, Deserialize)]
pub struct PairingInfo {
    pub version: u8,
    pub public_key: String,      // base64 server pubkey
    pub endpoint: String,         // host:port
    pub pairing_token: String,    // one-time auth token
    pub api_port: u16,
}

impl RemoteServer {
    pub fn new() -> Self {
        // Generate WireGuard keypair
        let secret = x25519_dalek::StaticSecret::random_from_rng(rand::thread_rng());
        let public = x25519_dalek::PublicKey::from(&secret);

        Self {
            private_key: secret.to_bytes(),
            public_key: public.to_bytes(),
            listen_port: 51820,
            devices: Vec::new(),
            running: false,
        }
    }

    /// Generate pairing info for QR code
    pub fn generate_pairing(&self, endpoint: &str) -> PairingInfo {
        let token: String = (0..6).map(|_| {
            let n: u8 = rand::random::<u8>() % 10;
            (b'0' + n) as char
        }).collect();

        PairingInfo {
            version: 1,
            public_key: base64::engine::general_purpose::STANDARD.encode(self.public_key),
            endpoint: endpoint.to_string(),
            pairing_token: token,
            api_port: 8443,
        }
    }

    /// Generate QR code string from pairing info
    pub fn generate_qr_string(&self, endpoint: &str) -> String {
        let info = self.generate_pairing(endpoint);
        serde_json::to_string(&info).unwrap_or_default()
    }

    /// Register a new device
    pub fn register_device(&mut self, name: &str, pubkey: [u8; 32]) -> String {
        let ip_num = self.devices.len() + 2; // .1 = server, .2+ = clients
        let ip = format!("10.99.0.{}", ip_num);

        self.devices.push(PairedDevice {
            name: name.to_string(),
            public_key: pubkey,
            internal_ip: ip.clone(),
            permissions: DevicePermissions::default(),
            paired_at: chrono_now(),
            last_seen: None,
        });

        ip
    }

    /// Revoke a device by name
    pub fn revoke_device(&mut self, name: &str) {
        self.devices.retain(|d| d.name != name);
    }

    /// List paired devices
    pub fn list_devices(&self) -> &[PairedDevice] {
        &self.devices
    }
}

fn chrono_now() -> String {
    // Simple timestamp without chrono dependency
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{}", secs)
}

use base64::Engine;

/// API routes for the remote server (axum)
pub mod api {
    use super::*;
    use axum::{routing::{get, post}, Router, Json};
    use std::sync::{Arc, Mutex};

    pub fn routes(server: Arc<Mutex<RemoteServer>>) -> Router {
        Router::new()
            .route("/api/status", get({
                let s = server.clone();
                move || async move {
                    let srv = s.lock().unwrap();
                    Json(serde_json::json!({
                        "running": srv.running,
                        "devices": srv.devices.len(),
                        "port": srv.listen_port,
                    }))
                }
            }))
            .route("/api/devices", get({
                let s = server.clone();
                move || async move {
                    let srv = s.lock().unwrap();
                    Json(serde_json::json!({ "devices": srv.devices }))
                }
            }))
            .route("/api/pair", post({
                let s = server.clone();
                move |Json(body): Json<PairRequest>| async move {
                    let mut srv = s.lock().unwrap();
                    let mut key = [0u8; 32];
                    if let Ok(decoded) = base64::engine::general_purpose::STANDARD.decode(&body.public_key) {
                        if decoded.len() == 32 {
                            key.copy_from_slice(&decoded);
                        }
                    }
                    let ip = srv.register_device(&body.device_name, key);
                    Json(serde_json::json!({ "ok": true, "internal_ip": ip }))
                }
            }))
    }

    #[derive(Deserialize)]
    pub struct PairRequest {
        pub device_name: String,
        pub public_key: String, // base64
        pub pairing_token: String,
    }
}
