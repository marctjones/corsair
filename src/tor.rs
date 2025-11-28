/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

//! Tor client initialization using Arti

use arti_client::{TorClient, TorClientConfig};
use log::info;
use tor_rtcompat::tokio::TokioRuntimeHandle;

/// Create and bootstrap a Tor client
pub async fn create_tor_client() -> anyhow::Result<TorClient<TokioRuntimeHandle>> {
    info!("Creating Tor client configuration...");

    // Use default configuration
    let config = TorClientConfig::default();

    // Get current Tokio runtime handle
    let runtime = TokioRuntimeHandle::current()
        .expect("Must be called from within a Tokio runtime");

    info!("Bootstrapping Tor client (this may take a minute)...");

    // Create and bootstrap the client
    let client = TorClient::with_runtime(runtime)
        .config(config)
        .create_bootstrapped()
        .await?;

    info!("Tor client ready");
    Ok(client)
}
