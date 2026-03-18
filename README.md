# punchline

End-to-end encrypted peer-to-peer chat over UDP. No accounts, no central server relaying/storing messages, no middleman. Just two peers, a direct connection, and Noise protocol encryption.

https://github.com/user-attachments/assets/939e96d3-45e3-4484-9a27-28c3a0457b05


## Table of Contents

- [What It Does](#what-it-does)
- [Quick Start](#quick-start)
- [How It Works](#how-it-works)
- [CLI Reference](#cli-reference)
- [Usage](#usage)
  - [Configuration](#configuration)
  - [Managing Peers](#managing-peers)
  - [Status Check](#status-check)
  - [Server Options](#server-options)
  - [Theming](#theming)
  - [Shell Completions](#shell-completions)
- [Cryptography](#cryptography)
  - [IK Handshake](#ik-handshake)
  - [Initiator Determination](#initiator-determination)
  - [Key Storage](#key-storage)
- [Wire Protocol](#wire-protocol)
  - [Hole Punch Protocol](#hole-punch-protocol)
  - [Transport Protocol](#transport-protocol)
  - [Signal Protocol](#signal-protocol)
  - [STUN Protocol](#stun-protocol)
- [Project Structure](#project-structure)
- [Building from Source](#building-from-source)
- [Running Tests](#running-tests)
- [Tech Stack](#tech-stack)
- [License](#license)

---

## What It Does

Two people run `punchline connect <peer>` on their machines. Punchline punches through their NATs, performs an encrypted handshake, and drops them into a private chat - all in a few milliseconds. The included STUN and signal servers handle discovery, then get out of the way.
<img width="1422" height="920" alt="convo" src="https://github.com/user-attachments/assets/bceda761-5398-42ee-a706-9a7697140336" />


---

## Quick Start

```bash
cargo build --release
```

**Start the servers** (on a machine both peers can reach), or use my public ones hosted at `64.225.107.28` (STUN: port `3478`, signaling: port `8743`):

```bash
punchline-stund                  # STUN server - tells peers their public IP
punchline-signald                # Signal server - matches peers who want to talk
```
<img width="2540" height="1532" alt="servers" src="https://github.com/user-attachments/assets/81745412-6477-4119-a53c-3fc82889e414" />


**On each peer's machine:**

```bash
# Generate your identity (X25519 keypair)
punchline-client keygen

# Share your public key with your peer
punchline-client pubkey

# Save their key
punchline-client peers add alice a1b2c3d4...64_hex_chars

# Connect (both peers run this, targeting each other)
punchline-client connect alice --stun <server>:3478 --signal <server>:8743
```

The TUI launches with a live connection progress view:

1. STUN discovery - resolving your external address via `punchline-stund`
2. Signal server - connecting to `punchline-signald`
3. Waiting for peer - signal server matches both peers
4. Hole punch - establishing the direct UDP path
5. Noise handshake - encrypted key exchange

   <img width="1908" height="1228" alt="dashboard" src="https://github.com/user-attachments/assets/89a84985-c83b-4a72-bfbb-63b6890ff279" />


Once complete, you're in the chat. Type and press Enter. Press Esc to quit.

---

## How It Works

The entire system consists of three binaries, all included in this repo:

| Binary | Role | When used |
|---|---|---|
| `punchline-stund` | STUN server (UDP) - responds with the client's external IP:port | During setup only |
| `punchline-signald` | Signal server (WebSocket) - matches peers and exchanges addresses | During setup only |
| `punchline-client` | The messenger itself - CLI, TUI, crypto, hole punching | Always |

After the initial setup, the STUN and signal servers are no longer contacted. Everything flows directly peer-to-peer.
<!-- TODO: Diagram -->

---

## CLI Reference

### `punchline-client`

| Command | Description |
|---|---|
| `keygen [--force] [-i path]` | Generate a new X25519 identity keypair. Use `--force` to overwrite without prompting. Use `-i` to specify output path. |
| `pubkey [-i path]` | Print your public key (64 hex characters). Use `-i` to derive from a specific key file. |
| `connect <peer> [-i path] [--stun addr] [--signal addr]` | Connect to a peer by alias or raw hex key. Use `-i` to specify identity key. Launches the TUI. |
| `peers` | List all known peers. |
| `peers add <name> <key>` | Save a peer's public key under a nickname. |
| `peers remove <name>` | Remove a peer by nickname. |
| `config path` | Print the config file path. |
| `config show` | Show current configuration values. |
| `status` | Show identity, config, server reachability, and peer count. |
| `completions <shell>` | Generate shell completions (`bash`, `zsh`, or `fish`). |

**Global flags:**

| Flag | Description |
|---|---|
| `-v` | Increase log verbosity (`-v` = debug, `-vv` = trace). |
| `-q, --quiet` | Suppress all log output. |

### `punchline-stund`

| Flag | Description |
|---|---|
| `--address <addr>` | Bind address (default: `0.0.0.0`). |
| `--port <port>` | Bind port (default: `3478`). |
| `-v / -vv` | Debug / trace logging. |
| `-q` | Quiet mode. |

### `punchline-signald`

| Flag | Description |
|---|---|
| `--address <addr>` | Bind address (default: `0.0.0.0`). |
| `--port <port>` | Bind port (default: `8743`). |
| `-v / -vv` | Debug / trace logging. |
| `-q` | Quiet mode. |

---

## Usage

### Configuration

Instead of passing `--stun` and `--signal` every time, create `~/.config/punchline/config.toml`:

```toml
stun_server = "203.0.113.10:3478"
signal_server = "203.0.113.10:8743"
```

### Managing Peers

```bash
punchline-client peers                              # list all
punchline-client peers add alice a1b2c3d4...        # add
punchline-client peers remove alice                 # remove
```

Aliases are stored in `~/.punchline/known_peers.toml`. You can also connect with a raw 64-char hex key directly.

### Status Check

```bash
punchline-client status
```

Shows your identity, config, server reachability (sends a real STUN probe and TCP connect), and peer count.

### Server Options

Both servers support `-v` (debug), `-vv` (trace), `-q` (quiet), `--address`, and `--port`:

```bash
punchline-stund -v --port 3478
punchline-signald -v --port 8743
```

### Theming

Customize the TUI via `~/.config/punchline/style.toml`
Styles used in the video:

```toml
[colors]
my_text = "#ebdbb2"
peer_text = "#bdae93"
input_text = "#ebdbb2"
border = "#ebdbb2"
sidebar_key = "#ebdbb2"
sidebar_value = "#bdae93"

[padding]
chat_horizontal = 2
chat_vertical = 1
```

All colors are hex RGB. If the file is absent, the terminal's default colors are used.

### Shell Completions

```bash
punchline-client completions bash > ~/.local/share/bash-completion/completions/punchline-client
punchline-client completions zsh > ~/.zfunc/_punchline-client
punchline-client completions fish > ~/.config/fish/completions/punchline-client.fish
```

---

## Cryptography

Full protocol name: `Noise_IK_25519_ChaChaPoly_SHA256`

| Component | Role |
|---|---|
| **Noise IK** | Handshake pattern - initiator knows responder's public key. Completes in 2 messages. |
| **X25519** | Elliptic-curve Diffie-Hellman key exchange (RFC 7748). 128-bit security, constant-time. |
| **ChaCha20-Poly1305** | AEAD cipher for message encryption (RFC 8439). Same cipher used in TLS 1.3 and WireGuard. |
| **SHA-256** | Used internally by Noise for key derivation and handshake hashing. |

### IK Handshake

The IK pattern means the initiator knows the responder's static public key before the handshake begins. Both peers already have each other's keys (exchanged out-of-band or via the peer registry), so no trust-on-first-use is required.

1. **Initiator -> Responder**: Sends an encrypted message containing its static public key, encrypted under the responder's known key. Provides identity hiding against passive observers.
2. **Responder -> Initiator**: Decrypts, verifies, and replies. Both sides transition to transport mode with shared session keys.

### Initiator Determination

Punchline deterministically selects the initiator by comparing the first 8 bytes of each peer's public key as a big-endian `u64`. The peer with the smaller value becomes the initiator. Both sides compute this independently.

### Key Storage

The identity is a 32-byte X25519 secret key at `~/.punchline/id_x25519` with Unix permissions `0600`. The public key is derived on load. Key generation uses `x25519-dalek` with `OsRng`.

---

## Wire Protocol

The first byte of each UDP packet identifies its type:

| Prefix | Type      | Phase      | Description                           |
|--------|-----------|------------|---------------------------------------|
| `0x00` | PROBE     | Hole punch | Sent every 200ms to open NAT pinhole  |
| `0x01` | ACK       | Hole punch | Confirms receipt of a PROBE           |
| (none) | Handshake | Handshake  | Raw Noise-encrypted handshake payload |
| `0x02` | Message   | Transport  | Encrypted chat message                |
| `0x03` | Keepalive | Transport  | Encrypted empty payload (heartbeat)   |

### Hole Punch Protocol

Both peers execute the same algorithm simultaneously:

1. Send `PROBE` (0x00) every 200ms to the peer's external address.
2. On receiving a `PROBE`, switch to sending `ACK` (0x01).
3. On receiving an `ACK`, send one final `ACK` and declare success.
4. Safety timeout: 2 seconds of sending ACKs without reply assumes the peer finished.

### Transport Protocol

Messages (`0x02`) carry Noise-encrypted UTF-8 payloads. Keepalives (`0x03`) are encrypted empty payloads sent every 10 seconds to maintain cipher nonce synchronization. 30 seconds without any packet triggers disconnect.

### Signal Protocol

JSON over WebSocket:

```json
// PairRequest (client -> server)
{ "external_addr": "203.0.113.5:48291", "public_key": "a1b2...", "target_public_key": "d4e5..." }

// PairResponse (server -> client)
{ "target_external_addr": "198.51.100.7:51003", "target_public_key": "d4e5..." }
```

### STUN Protocol

Follows RFC 5389 (simplified): binding request/response with `XOR-MAPPED-ADDRESS`. IPv4 only.

---

## Project Structure

Cargo workspace with four crates:

```
crates/
├── proto/      # Shared library: crypto, STUN, signal types, transport trait
├── client/     # P2P client: CLI, TUI, connection logic, peer management
├── signald/    # Signal server: WebSocket peer matching
└── stund/      # STUN server: external address discovery
```

---

## Building from Source

**Prerequisites:** Rust 2024 edition (rustc 1.85+)

```bash
git clone https://github.com/notmichaelpielka/punchline.git
cd punchline
cargo build --release
```

Binaries are placed in `target/release/`:
- `punchline-client`
- `punchline-signald`
- `punchline-stund`

---

## Running Tests

```bash
cargo test
```

Tests cover cryptographic operations, STUN encoding/decoding, signal protocol serialization, config parsing, peer management, style theming, and the Noise IK handshake.

---

## Tech Stack

| Crate | Purpose |
|---|---|
| [`snow`](https://crates.io/crates/snow) | Noise protocol framework (handshake + transport encryption) |
| [`x25519-dalek`](https://crates.io/crates/x25519-dalek) | X25519 key generation and derivation |
| [`ratatui`](https://crates.io/crates/ratatui) | Terminal UI framework |
| [`crossterm`](https://crates.io/crates/crossterm) | Terminal event handling |
| [`clap`](https://crates.io/crates/clap) | CLI argument parsing + shell completions |
| [`tungstenite`](https://crates.io/crates/tungstenite) | WebSocket client/server |
| [`tracing`](https://crates.io/crates/tracing) | Structured logging |

---

## License

MIT - see [LICENSE](LICENSE).
