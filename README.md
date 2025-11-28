# Corsair 🏴‍☠️

Embedded Tor proxy daemon for Servo browser using Arti over Unix Domain Sockets.

## Why a Separate Binary?

Arti (Rust Tor implementation) and Stylo (Servo's CSS engine) cannot coexist in the same Cargo workspace due to Rust compiler trait recursion overflow. Corsair runs as a separate process:

- **Corsair**: Separate workspace with Arti, provides Tor connectivity via UDS
- **Compass**: Browser with Stylo, connects to Corsair via UDS

From the user's perspective, this is transparent.

## Architecture

```
┌──────────────┐                      ┌─────────────┐
│   Compass    │  Unix Domain Socket  │   Corsair   │
│   Browser    │◄────────────────────►│  Tor Daemon │
│   (Stylo)    │   Binary Protocol    │   (Arti)    │
└──────────────┘                      └──────┬──────┘
                                            │
                                            ▼
                                       Tor Network
```

## IPC Protocol

Corsair uses a simple binary protocol over UDS (not SOCKS5):

1. **Client sends** `ConnectRequest` (host, port)
2. **Server responds** `ConnectResponse` (success/error)
3. **If successful**, bidirectional data relay begins

### Message Format

```
[4 bytes: length (big-endian)][bincode-serialized data]
```

### ConnectRequest
```rust
struct ConnectRequest {
    host: String,  // Target hostname (can be .onion)
    port: u16,     // Target port
}
```

### ConnectResponse
```rust
struct ConnectResponse {
    success: bool,
    error: Option<String>,
}
```

## Building

```bash
cd corsair
cargo build --release
```

Binary output: `target/release/corsair`

## Running

### Standalone
```bash
corsair [/path/to/socket]
```

Default socket: `/tmp/servo-sockets/corsair.sock`

### With Compass Browser
Compass automatically launches Corsair - no manual intervention needed.

## Features

- **Pure Rust**: Uses Arti, the official Rust Tor implementation
- **Simple IPC**: Binary protocol over UDS (no SOCKS5 complexity)
- **Secure**: Socket with user-only permissions (0600)
- **Auto-launch**: Compass starts Corsair automatically
- **Process Isolation**: Separate from browser for stability

## Client Usage (from Rigging library)

```rust
use tokio::net::UnixStream;

// Connect to Corsair
let mut stream = UnixStream::connect("/tmp/servo-sockets/corsair.sock").await?;

// Send connect request
let request = ConnectRequest {
    host: "example.onion".to_string(),
    port: 80,
};
send_request(&mut stream, &request).await?;

// Read response
let response = read_response(&mut stream).await?;
if response.success {
    // Now relay HTTP data through the stream
    // ...
}
```

## Logging

```bash
# Info level (default)
corsair

# Debug level
RUST_LOG=debug corsair

# Trace level (very verbose)
RUST_LOG=trace corsair
```

## Security

- **UDS Permissions**: Socket created with 0600 (user-only)
- **No Network Exposure**: Socket only accessible locally
- **Process Isolation**: Runs as separate process from browser
- **Arti Security**: Inherits Arti's Tor protocol security

## Performance

- **Low Latency**: UDS faster than TCP loopback
- **No SOCKS Overhead**: Direct binary protocol
- **Async**: Tokio for concurrent connections
- **Memory**: ~50MB resident when idle

## Troubleshooting

### Socket already in use
```bash
rm /tmp/servo-sockets/corsair.sock
```

### Permission denied
```bash
ls -l /tmp/servo-sockets/corsair.sock
# Should be: srw------- (600)
```

### Tor not bootstrapping
```bash
RUST_LOG=debug corsair 2>&1 | tee corsair.log
```

## License

Mozilla Public License 2.0 (MPL-2.0)

## Related Projects

- [Compass](https://github.com/marctjones/compass) - Privacy-focused browser
- [Harbor](https://github.com/marctjones/harbor) - Local app framework
- [Rigging](https://github.com/marctjones/rigging) - Transport library
