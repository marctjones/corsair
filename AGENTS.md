# AI Agent Development Guide for Corsair

This document provides instructions for AI coding assistants (Claude Code, Gemini, Cursor, etc.) working on the Corsair Tor daemon.

## Project Overview

**Corsair** is an embedded Tor proxy daemon for Servo-based browsers. It provides:
- Tor connectivity via Arti (Rust Tor implementation)
- Binary IPC protocol over Unix Domain Sockets
- Process isolation from the main browser (avoids Arti/Stylo conflicts)
- Lightweight daemon design

## Why Corsair Exists

Arti (Tor in Rust) and Servo's Stylo (CSS engine) have conflicting trait implementations that cause Rust compiler recursion overflow when compiled together. Corsair solves this by:
1. Running as a separate process
2. Communicating via binary IPC over Unix sockets
3. Providing a simple request/response protocol

**Important**: Do NOT try to embed Arti directly in Servo - it will fail with recursion limit errors.

## Repository Structure

```
corsair/
├── Cargo.toml           # Package manifest (isolated workspace)
├── src/
│   ├── main.rs          # Daemon entry point
│   ├── ipc.rs           # IPC protocol handling
│   └── tor.rs           # Arti/Tor integration
├── README.md
├── DESIGN.md
└── IMPLEMENTATION_PLAN.md
```

## Coding Standards

### Rust Guidelines
- **Edition**: Rust 2021
- **Async Runtime**: Tokio (required by Arti)
- **Error Handling**: `anyhow` for application errors
- **Serialization**: `bincode` for IPC messages
- **Logging**: `tracing` crate (Arti's choice)

### Code Style
```rust
// Good: Clear error context
async fn connect_tor(host: &str, port: u16) -> anyhow::Result<TorStream> {
    let client = TOR_CLIENT.get()
        .ok_or_else(|| anyhow!("Tor client not initialized"))?;

    client.connect((host, port))
        .await
        .context("Failed to establish Tor connection")
}

// Good: Proper IPC message handling
async fn handle_message(msg: IpcMessage) -> IpcResponse {
    match msg {
        IpcMessage::Connect { host, port } => {
            match connect_tor(&host, port).await {
                Ok(stream) => IpcResponse::Connected,
                Err(e) => IpcResponse::Error(e.to_string()),
            }
        }
        IpcMessage::NewIdentity => {
            // Request new Tor circuit
        }
    }
}
```

### Binary IPC Protocol

**Message Format:**
```
┌──────────────────┬─────────────────────────────┐
│ Length (4 bytes) │ Bincode-encoded payload     │
│    (u32 LE)      │                             │
└──────────────────┴─────────────────────────────┘
```

**Request Types:**
```rust
#[derive(Serialize, Deserialize)]
pub enum Request {
    Connect { host: String, port: u16 },
    NewIdentity,
    Status,
    Shutdown,
}
```

**Response Types:**
```rust
#[derive(Serialize, Deserialize)]
pub enum Response {
    Connected,
    Error(String),
    Status { circuits: u32, bootstrap: u8 },
    Ok,
}
```

## Key Concepts

### Why NOT SOCKS5?
- SOCKS5 adds overhead and complexity
- Binary IPC is faster and simpler
- Direct control over Tor features (new identity, etc.)
- Better error reporting

### Process Lifecycle
1. Browser starts Corsair daemon
2. Corsair bootstraps Tor network
3. Browser connects to Corsair socket
4. Browser sends connection requests
5. Corsair establishes Tor connections
6. Data relayed bidirectionally
7. Browser sends shutdown on exit

### Socket Path
Default: `/tmp/corsair.sock`
Configurable via `--socket` argument

## Development Tasks

### Adding a New IPC Command

1. Add variant to `Request` enum in `ipc.rs`
2. Add variant to `Response` enum
3. Implement handler in `handle_request()`
4. Update Rigging's `TorConnector` to use it
5. Write tests
6. Update documentation

### Modifying Tor Behavior

1. Check Arti documentation: https://docs.rs/arti-client
2. Modify `tor.rs` initialization
3. Test with actual Tor network
4. Consider bootstrap time impact

## Common Commands

```bash
# Build
cargo build --release

# Run daemon
./target/release/corsair --socket /tmp/corsair.sock

# Run with verbose logging
RUST_LOG=debug ./target/release/corsair

# Test IPC (manual)
# Use a test client that speaks the binary protocol

# Check Arti version
cargo tree | grep arti
```

## Arti Integration

### Client Initialization
```rust
use arti_client::{TorClient, TorClientConfig};
use tor_rtcompat::tokio::TokioRuntimeHandle;

async fn init_tor() -> anyhow::Result<TorClient<TokioRuntimeHandle>> {
    let config = TorClientConfig::default();
    let runtime = TokioRuntimeHandle::current();

    TorClient::create_bootstrapped(runtime, config).await
}
```

### Connection Handling
```rust
async fn connect(client: &TorClient<impl Runtime>, host: &str, port: u16)
    -> anyhow::Result<DataStream>
{
    let stream = client.connect((host, port)).await?;
    Ok(stream)
}
```

## Important Notes

1. **Isolated Workspace**: Corsair must remain in its own workspace to avoid Stylo conflicts
2. **Bootstrap Time**: Tor bootstrap takes 10-60 seconds on first run
3. **Circuit Reuse**: Multiple connections may share circuits for performance
4. **Memory Usage**: Arti uses ~50-100MB RAM
5. **No onion services yet**: Client-only for now

## Error Scenarios

| Error | Cause | Solution |
|-------|-------|----------|
| Bootstrap timeout | Network issues | Check connectivity, increase timeout |
| Socket permission denied | Wrong permissions | Check socket file permissions |
| Address already in use | Daemon already running | Kill existing process |
| Recursion limit | Compiled with Servo | Must be separate workspace |

## Testing

```bash
# Unit tests
cargo test

# Integration test (requires network)
cargo test --features integration

# Manual testing
# 1. Start daemon
./target/release/corsair &

# 2. Use test client or Rigging's TorConnector
```

## Security Considerations

1. Socket has user-only permissions (0600)
2. Daemon runs with minimal privileges
3. No sensitive data logged
4. Tor directory stored in user's home

## Related Projects

- [Arti](https://gitlab.torproject.org/tpo/core/arti) - Rust Tor implementation
- [Rigging](https://github.com/marctjones/rigging) - Transport library (uses Corsair)
- [Compass](https://github.com/marctjones/compass) - Browser (starts Corsair)
