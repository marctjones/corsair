# Corsair Design Document

## Overview

Corsair is an embedded Tor proxy daemon designed for Servo-based browsers. It provides Tor connectivity through a binary IPC protocol over Unix Domain Sockets.

## Problem Statement

### The Arti/Stylo Conflict

When Arti (Rust Tor implementation) and Stylo (Servo's CSS engine) are compiled in the same workspace, the Rust compiler encounters a trait recursion overflow:

```
error[E0275]: overflow evaluating the requirement
  `<... as std::iter::Iterator>::Item == ...`
```

This is due to complex generic bounds in both libraries that exceed the compiler's recursion limit. Increasing `RUST_RECURSION_LIMIT` does not solve the problem as it's a fundamental compilation issue.

### Solution: Process Isolation

Corsair solves this by running Arti in a separate process:

```
┌─────────────────┐     IPC      ┌─────────────────┐
│  Compass/Servo  │◄────────────►│     Corsair     │
│  (with Stylo)   │  Unix Socket │   (with Arti)   │
└─────────────────┘              └─────────────────┘
```

## Goals

1. **Process Isolation**: Keep Arti separate from Servo
2. **Minimal Overhead**: Efficient IPC protocol
3. **Simple Integration**: Easy for browsers to use
4. **Feature Parity**: Support key Tor features

## Non-Goals

1. Not a general-purpose SOCKS proxy
2. Not an onion service host (client only)
3. Not a system-wide Tor daemon

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Corsair Daemon                          │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────────┐         ┌──────────────────────────┐  │
│  │   IPC Handler    │         │     Tor Manager          │  │
│  │                  │         │                          │  │
│  │  - Accept conn   │         │  - Arti client           │  │
│  │  - Parse request │────────►│  - Connection pool       │  │
│  │  - Send response │         │  - Circuit management    │  │
│  │  - Relay data    │◄────────│  - Bootstrap state       │  │
│  └──────────────────┘         └──────────────────────────┘  │
│           ▲                                                  │
│           │                                                  │
│  ┌────────┴─────────┐                                       │
│  │  Unix Socket     │                                       │
│  │  Listener        │                                       │
│  └──────────────────┘                                       │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## IPC Protocol

### Why Binary IPC (Not SOCKS5)

| Aspect | Binary IPC | SOCKS5 |
|--------|-----------|--------|
| Overhead | Minimal | Protocol negotiation |
| Features | Full control | Limited to SOCKS spec |
| Error reporting | Rich errors | Generic codes |
| New identity | Native support | Requires control port |
| Complexity | Simple | Multiple handshakes |

### Message Format

```
┌──────────────────┬─────────────────────────────────┐
│ Length (4 bytes) │ Bincode-encoded message         │
│ u32 little-end   │                                 │
└──────────────────┴─────────────────────────────────┘
```

### Request Messages

```rust
#[derive(Serialize, Deserialize)]
pub enum Request {
    /// Connect to a host through Tor
    Connect {
        host: String,
        port: u16,
    },

    /// Request a new Tor identity (new circuits)
    NewIdentity,

    /// Get daemon status
    Status,

    /// Graceful shutdown
    Shutdown,
}
```

### Response Messages

```rust
#[derive(Serialize, Deserialize)]
pub enum Response {
    /// Connection established - socket now relays data
    Connected,

    /// Error occurred
    Error {
        code: ErrorCode,
        message: String,
    },

    /// Status response
    Status {
        bootstrap_percent: u8,
        circuit_count: u32,
        version: String,
    },

    /// Command acknowledged
    Ok,
}

#[derive(Serialize, Deserialize)]
pub enum ErrorCode {
    NotBootstrapped,
    ConnectionFailed,
    HostUnreachable,
    Timeout,
    InternalError,
}
```

### Connection Flow

```
Browser                          Corsair
   │                                │
   │──── Connect {host, port} ─────►│
   │                                │ (establish Tor connection)
   │◄─────── Connected ─────────────│
   │                                │
   │◄═══════ Data relay ═══════════►│
   │                                │
   │ (connection closed)            │
```

## Tor Integration

### Bootstrap Process

1. Daemon starts
2. Initialize Arti client with default config
3. Begin Tor bootstrap (connect to network)
4. Accept IPC connections (may queue requests)
5. Process requests once bootstrapped

### Circuit Management

- Circuits are managed by Arti automatically
- New identity request clears circuit cache
- Multiple connections may share circuits

### Configuration

```rust
pub struct CorsairConfig {
    /// Unix socket path
    pub socket_path: String,

    /// Tor data directory
    pub tor_data_dir: PathBuf,

    /// Bootstrap timeout
    pub bootstrap_timeout: Duration,

    /// Enable onion service connections
    pub enable_onion: bool,
}
```

## Error Handling

### Bootstrap Errors

```rust
enum BootstrapError {
    NetworkUnreachable,
    DirectoryServerFailure,
    Timeout,
    ConfigurationError,
}
```

### Connection Errors

```rust
enum ConnectionError {
    HostNotFound,
    ConnectionRefused,
    Timeout,
    CircuitBuildFailed,
    StreamFailed,
}
```

## Security Considerations

### Socket Security

- Socket created with mode 0600 (user only)
- Socket in `/tmp` or user-specified location
- Validates connecting process (future: SO_PEERCRED)

### Data Handling

- No logging of connection destinations
- No persistent storage of traffic data
- Memory cleared on shutdown

### Process Isolation

- Runs with minimal privileges
- No network access except Tor
- Sandboxing support planned

## Performance

### Latency

- IPC overhead: ~0.1ms per message
- Tor connection: 500ms-3s (circuit building)
- Data relay: minimal overhead

### Memory

- Base: ~30MB
- With Tor client: ~50-100MB
- Per connection: ~10KB

### Connections

- Supports hundreds of concurrent connections
- Uses async I/O throughout
- Connection pooling for common destinations

## Platform Support

| Platform | Status |
|----------|--------|
| Linux | Supported |
| macOS | Supported |
| Windows | Planned (Named Pipes) |

## Future Extensions

1. **Onion Services**: Host .onion addresses
2. **Bridge Support**: Connect via bridges
3. **Pluggable Transports**: obfs4, meek, etc.
4. **Control API**: Runtime configuration
5. **Metrics**: Prometheus-compatible stats
