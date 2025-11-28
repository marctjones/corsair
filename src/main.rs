/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Corsair - Embedded Tor Proxy Daemon
//!
//! Provides Tor connectivity for Servo browser via Unix domain socket IPC.
//! Uses Arti (Rust Tor implementation) and a simple binary protocol for
//! connection requests.
//!
//! # IPC Protocol
//!
//! Instead of SOCKS5, Corsair uses a simple binary protocol:
//!
//! 1. Client sends ConnectRequest (host, port)
//! 2. Server responds with ConnectResponse (success/error)
//! 3. If successful, bidirectional data relay begins
//!
//! This avoids SOCKS5 complexity and is optimized for UDS IPC.

mod ipc;
mod tor;

use anyhow::Result;
use log::{error, info, warn};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::UnixListener;
use tokio::sync::Mutex;

/// Default socket path for Corsair
const DEFAULT_SOCKET_PATH: &str = "/tmp/servo-sockets/corsair.sock";

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .format_timestamp_millis()
        .init();

    info!("Corsair Tor Proxy Daemon v{}", env!("CARGO_PKG_VERSION"));

    // Get socket path from args or use default
    let socket_path = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(DEFAULT_SOCKET_PATH));

    // Ensure socket directory exists
    if let Some(parent) = socket_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Remove existing socket file
    if socket_path.exists() {
        std::fs::remove_file(&socket_path)?;
    }

    // Bootstrap Tor client
    info!("Bootstrapping Arti Tor client...");
    let tor_client = match tor::create_tor_client().await {
        Ok(client) => {
            info!("Tor client bootstrapped successfully");
            Arc::new(client)
        }
        Err(e) => {
            error!("Failed to bootstrap Tor client: {}", e);
            return Err(e.into());
        }
    };

    // Create Unix socket listener
    let listener = UnixListener::bind(&socket_path)?;

    // Set socket permissions (user-only)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&socket_path, std::fs::Permissions::from_mode(0o600))?;
    }

    info!("Listening on {}", socket_path.display());
    info!("Ready to accept connections");

    // Accept connections
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let tor = tor_client.clone();
                tokio::spawn(async move {
                    if let Err(e) = ipc::handle_connection(stream, tor).await {
                        warn!("Connection handler error: {}", e);
                    }
                });
            }
            Err(e) => {
                error!("Accept error: {}", e);
            }
        }
    }
}
