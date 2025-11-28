/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! IPC Protocol for Corsair
//!
//! Simple binary protocol for connection requests over Unix domain socket.
//!
//! # Message Format
//!
//! All messages are length-prefixed with a 4-byte big-endian length,
//! followed by bincode-serialized data.
//!
//! ## ConnectRequest
//! ```text
//! [4 bytes: length][ConnectRequest bincode]
//! ```
//!
//! ## ConnectResponse
//! ```text
//! [4 bytes: length][ConnectResponse bincode]
//! ```
//!
//! After successful connection, raw bidirectional data relay begins.

use arti_client::TorClient;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tor_rtcompat::tokio::TokioRuntimeHandle;

/// Request to connect to a remote host through Tor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectRequest {
    /// Target hostname (can be .onion)
    pub host: String,
    /// Target port
    pub port: u16,
}

/// Response to a connection request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectResponse {
    /// Whether the connection succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Handle an incoming IPC connection
pub async fn handle_connection(
    mut stream: UnixStream,
    tor_client: Arc<TorClient<TokioRuntimeHandle>>,
) -> anyhow::Result<()> {
    debug!("New IPC connection");

    // Read request length (4 bytes, big-endian)
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    if len > 1024 * 1024 {
        // Sanity check: max 1MB
        return Err(anyhow::anyhow!("Request too large: {} bytes", len));
    }

    // Read request data
    let mut request_buf = vec![0u8; len];
    stream.read_exact(&mut request_buf).await?;

    // Deserialize request
    let request: ConnectRequest = bincode::deserialize(&request_buf)?;
    info!("Connect request: {}:{}", request.host, request.port);

    // Attempt Tor connection
    match tor_client.connect((request.host.as_str(), request.port)).await {
        Ok(mut tor_stream) => {
            // Send success response
            let response = ConnectResponse {
                success: true,
                error: None,
            };
            send_response(&mut stream, &response).await?;

            info!("Connected to {}:{} via Tor", request.host, request.port);

            // Bidirectional relay
            let (mut client_read, mut client_write) = stream.into_split();
            let (mut tor_read, mut tor_write) = tor_stream.split();

            let client_to_tor = tokio::io::copy(&mut client_read, &mut tor_write);
            let tor_to_client = tokio::io::copy(&mut tor_read, &mut client_write);

            tokio::select! {
                result = client_to_tor => {
                    if let Err(e) = result {
                        debug!("Client to Tor relay ended: {}", e);
                    }
                }
                result = tor_to_client => {
                    if let Err(e) = result {
                        debug!("Tor to client relay ended: {}", e);
                    }
                }
            }

            debug!("Connection relay finished");
        }
        Err(e) => {
            error!("Tor connection failed: {}", e);

            // Send error response
            let response = ConnectResponse {
                success: false,
                error: Some(e.to_string()),
            };
            send_response(&mut stream, &response).await?;
        }
    }

    Ok(())
}

/// Send a response message
async fn send_response(stream: &mut UnixStream, response: &ConnectResponse) -> anyhow::Result<()> {
    let data = bincode::serialize(response)?;
    let len = (data.len() as u32).to_be_bytes();

    stream.write_all(&len).await?;
    stream.write_all(&data).await?;
    stream.flush().await?;

    Ok(())
}

/// Read a response message (used by client side)
pub async fn read_response(stream: &mut UnixStream) -> anyhow::Result<ConnectResponse> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf).await?;
    let len = u32::from_be_bytes(len_buf) as usize;

    let mut data = vec![0u8; len];
    stream.read_exact(&mut data).await?;

    let response: ConnectResponse = bincode::deserialize(&data)?;
    Ok(response)
}

/// Send a connection request (used by client side)
pub async fn send_request(stream: &mut UnixStream, request: &ConnectRequest) -> anyhow::Result<()> {
    let data = bincode::serialize(request)?;
    let len = (data.len() as u32).to_be_bytes();

    stream.write_all(&len).await?;
    stream.write_all(&data).await?;
    stream.flush().await?;

    Ok(())
}
