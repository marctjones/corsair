# Corsair Implementation Plan

## Phase 1: Core Daemon (Current)

### 1.1 Basic Structure
- [x] Daemon entry point
- [x] Command-line argument parsing
- [x] Socket creation and listening
- [ ] Graceful shutdown handling
- [ ] PID file management

### 1.2 IPC Protocol
- [x] Message format definition
- [x] Request/Response types
- [x] Bincode serialization
- [ ] Protocol version negotiation
- [ ] Message validation

### 1.3 Connection Handling
- [x] Accept incoming connections
- [x] Parse requests
- [x] Send responses
- [ ] Connection timeout handling
- [ ] Maximum connection limit

## Phase 2: Tor Integration

### 2.1 Arti Client Setup
- [x] Basic Arti initialization
- [ ] Custom configuration loading
- [ ] Data directory management
- [ ] Bootstrap progress reporting

### 2.2 Connection Management
- [x] Basic TCP connection through Tor
- [ ] Connection timeout configuration
- [ ] Retry logic
- [ ] Circuit isolation options

### 2.3 Bidirectional Relay
- [x] Basic data relay
- [ ] Buffer management
- [ ] Backpressure handling
- [ ] Connection cleanup

## Phase 3: Advanced Features

### 3.1 New Identity
- [ ] Implement new identity request
- [ ] Circuit cleanup
- [ ] Connection migration

### 3.2 Status Reporting
- [ ] Bootstrap percentage
- [ ] Circuit count
- [ ] Bandwidth stats
- [ ] Version information

### 3.3 Error Handling
- [ ] Detailed error codes
- [ ] Error recovery strategies
- [ ] Client notification

## Phase 4: Production Ready

### 4.1 Robustness
- [ ] Watchdog timer
- [ ] Auto-restart on failure
- [ ] Memory leak detection
- [ ] Resource limits

### 4.2 Logging
- [ ] Structured logging (tracing)
- [ ] Log levels
- [ ] Log rotation
- [ ] Privacy-safe logging

### 4.3 Configuration
- [ ] Config file support
- [ ] Environment variables
- [ ] Runtime reconfiguration

## Phase 5: Platform Support

### 5.1 Windows
- [ ] Named pipe support
- [ ] Service integration
- [ ] Platform-specific paths

### 5.2 macOS
- [ ] Launchd integration
- [ ] Keychain for credentials
- [ ] Sandbox profile

### 5.3 Linux
- [ ] Systemd unit file
- [ ] AppArmor/SELinux profiles
- [ ] Filesystem permissions

## Phase 6: Security Hardening

### 6.1 Process Security
- [ ] Drop privileges after bind
- [ ] Seccomp filtering
- [ ] Namespace isolation

### 6.2 Socket Security
- [ ] Peer credential checking
- [ ] Connection rate limiting
- [ ] Authentication (optional)

### 6.3 Audit
- [ ] Security review
- [ ] Penetration testing
- [ ] Code audit

## Milestones

### v0.1.0 - Basic Daemon
- Accept connections
- Forward through Tor
- Binary IPC protocol

### v0.2.0 - Robust Operation
- Proper error handling
- Graceful shutdown
- Status reporting

### v0.3.0 - Feature Complete
- New identity support
- Full configuration
- Production logging

### v0.4.0 - Cross-Platform
- Windows support
- Platform packaging
- Service integration

### v1.0.0 - Stable Release
- Security hardening
- Full documentation
- Performance optimization

## Technical Debt

1. **Error Types**: Need consolidated error handling
2. **Tests**: Integration tests require Tor network
3. **Mocking**: Need mock Tor client for unit tests

## Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| arti-client | 0.36.x | Tor client |
| tor-rtcompat | 0.36.x | Runtime compatibility |
| tokio | 1.x | Async runtime |
| bincode | 1.x | Serialization |
| tracing | 0.1.x | Logging |
| clap | 4.x | CLI parsing |

## Open Questions

1. Should we support multiple simultaneous Tor clients?
2. How to handle bridge configuration securely?
3. Should we implement a control socket for management?

## Testing Strategy

### Unit Tests
- IPC message parsing
- Configuration validation
- Error handling

### Integration Tests
- Full connection flow (requires network)
- Bootstrap process
- New identity

### Manual Tests
- Browser integration
- Long-running stability
- Resource usage

## Contributing

See AGENTS.md for AI assistant guidelines and coding standards.
