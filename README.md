# punchline

Punchline is a peer-to-peer encrypted messenger that operates over UDP. It uses the Noise protocol framework for end-to-end encryption, STUN for NAT traversal, and a lightweight signaling server for peer discovery. Once a connection is established, all communication happens directly between the two peers with no intermediary.

The client features a terminal-based user interface (TUI) built with ratatui, showing a real-time connection progress view during setup and a split-pane chat interface once connected.

<!-- TODO: Add a screenshot or GIF of the TUI chat view here -->
<!-- ![punchline chat](path/to/chat-screenshot.png) -->

---

## Table of Contents

- [How It Works](#how-it-works)
- [Cryptography](#cryptography)
- [Connection Flow](#connection-flow)
- [Wire Protocol](#wire-protocol)
- [Crate Overview](#crate-overview)
  - [punchline-proto](#punchline-proto)
  - [punchline-client](#punchline-client)
  - [punchline-signald](#punchline-signald)
  - [punchline-stund](#punchline-stund)
- [Usage](#usage)
  - [Building](#building)
  - [Generating an Identity](#generating-an-identity)
  - [Configuration](#configuration)
  - [Managing Peers](#managing-peers)
  - [Running the Servers](#running-the-servers)
  - [Connecting to a Peer](#connecting-to-a-peer)
  - [Status Check](#status-check)
  - [Shell Completions](#shell-completions)
  - [Theming](#theming)
- [Architecture Diagrams](#architecture-diagrams)

---

## How It Works

Punchline establishes direct, encrypted UDP connections between two peers who may be behind NATs or firewalls. The process involves four cooperating components:

1. **STUN server** (`punchline-stund`) -- A lightweight UDP server that tells each client what their external (public) IP address and port are, as seen from the internet. This is necessary because peers behind NAT do not know their own public-facing address.

2. **Signal server** (`punchline-signald`) -- A WebSocket-based rendezvous point. Both peers connect to it, announce who they want to talk to (identified by public key), and the server matches them up. Once matched, each peer receives the other's external address. The signal server never sees any message content and is only used during connection setup.

3. **UDP hole punching** (`punchline-client`) -- After both peers know each other's external address, they simultaneously send UDP probe packets to each other. This creates entries in their respective NAT tables, allowing the direct peer-to-peer UDP path to open. Once both sides confirm the path is open via an acknowledgment exchange, the hole punch is complete.

4. **Noise handshake and encrypted messaging** (`punchline-client`) -- With a direct UDP path established, the peers perform a Noise IK handshake to authenticate each other and derive session encryption keys. All subsequent messages are encrypted with ChaCha20-Poly1305.

---

## Cryptography

Punchline uses the **Noise Protocol Framework** with the **IK** handshake pattern. The full protocol name is:

```
Noise_IK_25519_ChaChaPoly_SHA256
```

This breaks down as follows:

### Noise IK Pattern

The IK pattern means the **initiator knows the responder's static public key** before the handshake begins. Both peers already have each other's public keys (exchanged out-of-band or via the peer registry), so no trust-on-first-use is required. The handshake completes in a single round trip (two messages):

1. **Initiator -> Responder**: The initiator sends an encrypted handshake message containing its static public key, encrypted under the responder's known public key. This provides identity hiding for the initiator against passive observers.

2. **Responder -> Initiator**: The responder decrypts, verifies, and sends back its own encrypted handshake message. After this, both sides transition to transport mode with a shared symmetric session key.

### Initiator Determination

Rather than requiring an explicit role assignment, punchline deterministically selects the initiator by comparing the first 8 bytes of each peer's public key, interpreted as a big-endian `u64`. The peer with the numerically smaller value becomes the initiator. Both peers compute this independently and arrive at the same result.

### X25519 (Curve25519 Diffie-Hellman)

All key exchange uses X25519 (RFC 7748). Each identity is a 32-byte X25519 secret key stored on disk, from which the corresponding 32-byte public key is derived. X25519 provides 128-bit security and is resistant to timing side-channel attacks due to its constant-time implementation.

Key generation uses `x25519-dalek` with the operating system's cryptographic random number generator (`OsRng`).

### ChaCha20-Poly1305 (AEAD Cipher)

After the Noise handshake, all message payloads are encrypted using ChaCha20-Poly1305 (RFC 8439). This is an authenticated encryption scheme that provides both confidentiality and integrity. ChaCha20 is a stream cipher and Poly1305 is a message authentication code. This combination is used in TLS 1.3 and WireGuard.

### SHA-256

SHA-256 is used internally by the Noise framework for key derivation and handshake hashing. It is not directly exposed to the application layer.

### Key Storage

The identity secret key is stored as 32 raw bytes at `~/.punchline/id_x25519` with Unix permissions `0600` (read/write only for the owner).

---

## Connection Flow

The full sequence of events when two peers connect:

```
   Peer A                    STUN Server                Signal Server                 Peer B
     |                           |                           |                           |
     |--- STUN Binding Req ----->|                           |                           |
     |<-- STUN Binding Resp -----|                           |                           |
     |   (learns external addr)  |                           |                           |
     |                           |                           |                           |
     |                           |                           |<--- STUN Binding Req -----|
     |                           |                           |---- STUN Binding Resp --->|
     |                           |                           |   (learns external addr)  |
     |                           |                           |                           |
     |--- PairRequest (WS) ---->>>>>>>>>>>>>>>>>>>>>>>>>---->|                           |
     |                           |                           |<--- PairRequest (WS) -----|
     |                           |                           |                           |
     |                           |              (match found)|                           |
     |<-- PairResponse (WS) ---<<<<<<<<<<<<<<<<<<<<<<<<<<<---|                           |
     |                           |                           |---- PairResponse (WS) --->|
     |                           |                           |                           |
     |---------------------- UDP Hole Punch ------------------------------------------->|
     |   PROBE (0x00) --------->>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> PROBE (0x00)     |
     |   PROBE (0x00) <<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<-- PROBE (0x00)     |
     |   ACK   (0x01) --------->>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>> ACK   (0x01)     |
     |   ACK   (0x01) <<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<<-- ACK   (0x01)     |
     |                           |                           |                           |
     |---------- Noise IK Handshake (2 messages) ---------------------------------->|
     |                           |                           |                           |
     |========== Encrypted P2P Messaging (ChaCha20-Poly1305) ======================|
```

After the Noise handshake completes, both peers enter transport mode. Messages and keepalives flow directly over UDP, encrypted end-to-end. The STUN and signal servers are no longer contacted.

---

## Wire Protocol

All communication after STUN discovery happens over UDP. The first byte of each UDP packet identifies its type:

| Prefix | Type      | Phase      | Description                           |
|--------|-----------|------------|---------------------------------------|
| `0x00` | PROBE     | Hole punch | Sent repeatedly to open NAT pinhole   |
| `0x01` | ACK       | Hole punch | Confirms receipt of a PROBE           |
| (none) | Handshake | Handshake  | Raw Noise-encrypted handshake payload |
| `0x02` | Message   | Transport  | Encrypted chat message                |
| `0x03` | Keepalive | Transport  | Encrypted empty payload (heartbeat)   |

### Hole Punch Protocol

Both peers execute the same algorithm simultaneously:

1. Send `PROBE` (0x00) packets every 200ms to the peer's external address.
2. When a `PROBE` is received, switch to sending `ACK` (0x01) packets.
3. When an `ACK` is received, send one final `ACK` back and declare the punch successful.
4. Safety timeout: if ACKs have been sent for 2 seconds without receiving one back, assume the peer finished and stop.

### Transport Protocol

Once the Noise handshake is complete:

- **Messages** are prefixed with `0x02`, followed by the Noise-encrypted payload containing the UTF-8 message text.
- **Keepalives** are prefixed with `0x03`, followed by an encrypted empty payload. Keepalives are sent every 10 seconds when no messages are being sent. They advance the Noise nonce counter on both sides, maintaining cipher synchronization.
- If no packet (message or keepalive) is received for 30 seconds, the peer is considered disconnected.

### Signal Server Protocol

The signal server uses WebSocket over TCP. Messages are JSON:

**PairRequest** (client -> server):
```json
{
  "external_addr": "203.0.113.5:48291",
  "public_key": "a1b2c3...64 hex chars",
  "target_public_key": "d4e5f6...64 hex chars"
}
```

**PairResponse** (server -> client):
```json
{
  "target_external_addr": "198.51.100.7:51003",
  "target_public_key": "d4e5f6...64 hex chars"
}
```

The server holds incoming requests in a pending map keyed by public key. When two peers reference each other (mutual match), both receive a response with the other's external address.

### STUN Protocol

The STUN implementation follows RFC 5389 (simplified). It supports:

- **Binding Request** (type `0x0001`): 20-byte header with magic cookie `0x2112A442` and a 12-byte random transaction ID.
- **Binding Response** (type `0x0101`): Contains an `XOR-MAPPED-ADDRESS` attribute (type `0x0020`) with the client's external IP and port XOR'd against the magic cookie for privacy on the wire.

Only IPv4 is currently supported.

---

## Crate Overview

The project is organized as a Cargo workspace with four crates:

```
punchline/
├── Cargo.toml                  # Workspace root
├── Cargo.lock
└── crates/
    ├── proto/                  # Shared protocol library
    ├── client/                 # P2P client with TUI
    ├── signald/                # Signaling server
    └── stund/                  # STUN server
```

---

### punchline-proto

The shared protocol library used by all other crates. Contains cryptographic primitives, STUN packet parsing/building, signal server message types, transport abstractions, and error definitions.

**Dependencies**: `x25519-dalek`, `rand_core`, `hex`, `serde`, `thiserror`

```
crates/proto/
├── Cargo.toml
└── src/
    ├── lib.rs              # Module re-exports
    ├── crypto.rs           # X25519 keypair generation
    ├── error.rs            # ProtoError enum (STUN/key errors)
    ├── signal.rs           # PairRequest and PairResponse types
    ├── stun.rs             # STUN header parsing, binding request/response
    ├── transport.rs        # Transport trait (abstract send/recv interface)
    ├── udp.rs              # UdpTransport (Transport impl over std UdpSocket)
    └── tcp.rs              # TCP transport (placeholder)
```

**`crypto.rs`** -- X25519 keypair generation and public key derivation using `OsRng`.

**`error.rs`** -- `ProtoError` enum covering STUN parsing failures and invalid key/hex errors.

**`signal.rs`** -- `PairRequest` and `PairResponse` structs for peer discovery, serializable to/from JSON.

**`stun.rs`** -- STUN binding request/response building and parsing, including XOR-MAPPED-ADDRESS handling.

**`transport.rs`** -- Abstract `Transport` trait (`send_to`, `recv_from`, `try_clone`, etc.) for swappable network backends.

**`udp.rs`** -- `UdpTransport` implementing the `Transport` trait over `std::net::UdpSocket`.

**`tcp.rs`** -- Placeholder for a future TCP transport.

---

### punchline-client

The main application. Contains the CLI, TUI, connection orchestration, peer management, and all client-side networking logic.

**Dependencies**: `snow` (Noise protocol), `x25519-dalek`, `tungstenite` (WebSocket), `ratatui`, `crossterm`, `clap`, `chrono`, `serde`, `toml`, `tracing`

```
crates/client/
├── Cargo.toml
└── src/
    ├── lib.rs                  # Module re-exports
    ├── main.rs                 # Entry point, CLI dispatch
    ├── cli.rs                  # Clap CLI argument definitions
    ├── config.rs               # Config file loading (~/.config/punchline/config.toml)
    ├── identity.rs             # Key generation and loading (~/.punchline/id_x25519)
    ├── peers.rs                # Peer alias registry (~/.punchline/known_peers.toml)
    ├── handshake.rs            # Noise IK handshake execution
    ├── punch.rs                # UDP hole punch protocol
    ├── stun.rs                 # STUN client (external address discovery)
    ├── signal.rs               # Signal server WebSocket client
    ├── message.rs              # Encrypted message send/recv loops
    ├── style.rs                # TUI theme loading (~/.config/punchline/style.toml)
    ├── commands/
    │   ├── mod.rs              # Command module declarations
    │   ├── keygen.rs           # 'keygen' command handler
    │   ├── pubkey.rs           # 'pubkey' command handler
    │   ├── config.rs           # 'config' command handler
    │   ├── peers.rs            # 'peers' command handler
    │   ├── status.rs           # 'status' command handler
    │   ├── connect.rs          # 'connect' command (main orchestrator)
    │   └── completions.rs      # Shell completions generator
    └── tui/
        ├── mod.rs              # TUI module re-exports
        ├── app.rs              # App state machine, phases, event loop
        ├── events.rs           # AppEvent enum, keyboard handling
        └── render/
            ├── mod.rs          # Render module declarations
            ├── chat.rs         # Chat view (messages + sidebar)
            └── connecting.rs   # Connection progress view (ASCII art + steps)
```

**`main.rs`** -- Entry point; parses CLI arguments and dispatches to command handlers.

**`cli.rs`** -- Clap CLI definitions: global flags (`-i`, `-v`, `-q`) and all subcommands.

**`config.rs`** -- Loads `stun_server` and `signal_server` from `~/.config/punchline/config.toml`.

**`identity.rs`** -- Generates, stores, and loads the X25519 keypair at `~/.punchline/id_x25519`.

**`peers.rs`** -- Manages the alias-to-public-key registry at `~/.punchline/known_peers.toml`.

**`handshake.rs`** -- Performs the Noise IK handshake, deterministically choosing initiator by public key comparison.

**`punch.rs`** -- UDP hole punch via PROBE/ACK exchange with a sender thread and receiver loop.

**`stun.rs`** -- STUN client that discovers the external address and returns the bound socket for reuse.

**`signal.rs`** -- WebSocket client that sends a `PairRequest` and waits for a `PairResponse`.

**`message.rs`** -- Spawns send/recv threads for Noise-encrypted messaging with 10s keepalives and 30s disconnect timeout.

**`style.rs`** -- Loads TUI color and padding customization from `~/.config/punchline/style.toml`.

**`commands/connect.rs`** -- Main orchestrator: spawns connection and terminal threads, driving the full STUN -> signal -> punch -> handshake -> chat sequence.

**`commands/status.rs`** -- Prints identity, config, server reachability, and peer count.

**`commands/completions.rs`** -- Generates shell completions via clap_complete.

**`tui/app.rs`** -- App state machine with two phases (Connecting progress view, Connected chat view) and the main event loop.

**`tui/events.rs`** -- `AppEvent` enum and keyboard input handling (Esc, Enter, Backspace, character input).

**`tui/render/chat.rs`** -- Renders the chat layout: message panel, input box, and a sidebar showing peer info and crypto details.

**`tui/render/connecting.rs`** -- Renders the ASCII art banner, 5-step progress panel, and identity/target info sidebar.

---

### punchline-signald

The signaling server that facilitates peer discovery. It is a standalone binary that accepts WebSocket connections, matches peers who want to talk to each other, and sends them each other's external addresses.

**Dependencies**: `tungstenite`, `clap`, `serde_json`, `tracing`, `punchline-proto`

```
crates/signald/
├── Cargo.toml
└── src/
    ├── lib.rs              # Module declaration
    ├── main.rs             # Server logic (TCP listener, WebSocket, matching)
    └── cli.rs              # CLI arguments (address, port, verbosity)
```

**`main.rs`** -- TCP/WebSocket listener that matches mutual `PairRequest`s and sends each peer the other's external address.

**`cli.rs`** -- CLI arguments: `--address` (default `0.0.0.0`), `--port` (default `8743`), `-v`, `-q`.

---

### punchline-stund

The STUN server for external address discovery. It is a standalone binary that responds to STUN binding requests with the client's externally visible IP and port.

**Dependencies**: `clap`, `tracing`, `punchline-proto`

```
crates/stund/
├── Cargo.toml
└── src/
    ├── lib.rs              # Module declaration
    ├── main.rs             # Server logic (UDP listener, STUN responses)
    └── cli.rs              # CLI arguments (address, port, verbosity)
```

**`main.rs`** -- Single-threaded UDP loop that responds to STUN binding requests with the sender's XOR-MAPPED-ADDRESS.

**`cli.rs`** -- CLI arguments: `--address` (default `0.0.0.0`), `--port` (default `3478`), `-v`, `-q`.

---

## Usage

### Building

Requires Rust (edition 2024). Build all crates:

```bash
cargo build --release
```

The binaries will be at:
- `target/release/punchline-client`
- `target/release/punchline-signald`
- `target/release/punchline-stund`

### Generating an Identity

Before connecting to anyone, generate an X25519 keypair:

```bash
punchline-client keygen
```

This creates `~/.punchline/id_x25519` containing your 32-byte secret key and prints your public key. Use `--force` to overwrite an existing key.

To view your public key later:

```bash
punchline-client pubkey
```

<!-- TODO: Add a screenshot of the keygen + pubkey output here -->
<!-- ![keygen output](path/to/keygen-screenshot.png) -->

### Configuration

Create the config file at `~/.config/punchline/config.toml`:

```toml
stun_server = "203.0.113.10:3478"
signal_server = "203.0.113.10:8743"
```

Both fields are optional -- you can also pass them as CLI flags (`--stun`, `--signal`) on each `connect` invocation.

View the config file path and current values:

```bash
punchline-client config path
punchline-client config show
```

### Managing Peers

Instead of typing 64-character hex keys every time, you can register peer aliases:

```bash
# Add a peer
punchline-client peers add alice a1b2c3d4e5f6...64_hex_chars

# List all known peers
punchline-client peers

# Remove a peer
punchline-client peers remove alice
```

Peer aliases are stored in `~/.punchline/known_peers.toml`.

<!-- TODO: Add a screenshot of the peers list output here -->
<!-- ![peers list](path/to/peers-screenshot.png) -->

### Running the Servers

Both peers need access to a STUN server and a signal server. You can run your own:

**STUN server:**

```bash
# Default: listens on 0.0.0.0:3478
punchline-stund

# Custom address and port
punchline-stund --address 0.0.0.0 --port 3478

# With debug logging
punchline-stund -v
```

**Signal server:**

```bash
# Default: listens on 0.0.0.0:8743
punchline-signald

# Custom address and port
punchline-signald --address 0.0.0.0 --port 8743

# With debug logging
punchline-signald -v
```

Both servers support `-v` (debug), `-vv` (trace), and `-q` (quiet) log levels.

<!-- TODO: Add a screenshot or GIF showing server startup logs here -->
<!-- ![server logs](path/to/server-logs-screenshot.png) -->

### Connecting to a Peer

Both peers must run the `connect` command, targeting each other's public key. The order does not matter -- the signal server will match them once both are connected.

**Using a peer alias:**

```bash
punchline-client connect alice
```

**Using a raw public key:**

```bash
punchline-client connect a1b2c3d4e5f6...64_hex_chars
```

**With explicit server addresses (overrides config):**

```bash
punchline-client connect alice --stun 203.0.113.10:3478 --signal 203.0.113.10:8743
```

**With a custom identity file:**

```bash
punchline-client -i /path/to/id_x25519 connect alice
```

Once both peers have issued `connect`, the TUI launches and shows a connection progress view with five steps:

1. STUN discovery -- resolving your external address
2. Signal server -- connecting to the rendezvous server
3. Waiting for peer -- waiting for the other peer to connect to the signal server
4. Hole punch -- establishing the direct UDP path
5. Noise handshake -- performing the encrypted key exchange

<!-- TODO: Add a screenshot or GIF of the connecting progress view here -->
<!-- ![connecting view](path/to/connecting-screenshot.png) -->

Once all steps complete, the view switches to the chat interface. Type a message and press Enter to send. Press Esc to disconnect and quit.

<!-- TODO: Add a screenshot or GIF of the chat view here -->
<!-- ![chat view](path/to/chat-screenshot.png) -->

<!-- TODO: Add a video showing the full connection flow (both peers) here -->
<!-- ![full demo](path/to/demo-video.mp4) -->

### Status Check

View a diagnostic summary of your setup:

```bash
punchline-client status
```

This shows:
- Your identity (public key or "not found")
- Config file location and whether it exists
- STUN server address and reachability (sends a test binding request)
- Signal server address and reachability (TCP connection test)
- Number of known peers

<!-- TODO: Add a screenshot of the status output here -->
<!-- ![status output](path/to/status-screenshot.png) -->

### Shell Completions

Generate completions for your shell:

```bash
# Bash
punchline-client completions bash > ~/.local/share/bash-completion/completions/punchline-client

# Zsh
punchline-client completions zsh > ~/.zfunc/_punchline-client

# Fish
punchline-client completions fish > ~/.config/fish/completions/punchline-client.fish
```

### Theming

The TUI appearance can be customized via `~/.config/punchline/style.toml`:

```toml
[colors]
border = "#585858"
my_text = "#87afd7"
peer_text = "#d7875f"
input_text = "#d0d0d0"
sidebar_key = "#808080"
sidebar_value = "#b0b0b0"

[padding]
chat_horizontal = 2
chat_vertical = 1
```

All color values are hex RGB strings (`#RRGGBB`). The padding values control the margin around the TUI panels in terminal cells. If the file is absent or any field is omitted, the terminal's default colors are used.

---

## Architecture Diagrams

### Network Topology

```
                    +-----------+
                    |   STUN    |
                    |  Server   |
                    | (UDP:3478)|
                    +-----+-----+
                          |
              STUN Req/Resp (UDP)
                          |
          +---------------+---------------+
          |                               |
     +----+----+                     +----+----+
     | Peer A  |                     | Peer B  |
     | (Client)|                     | (Client)|
     +----+----+                     +----+----+
          |                               |
          |  PairReq/Resp (WebSocket)     |
          +-------+               +-------+
                  |               |
            +-----+-----+  +-----+-----+
            |  Signal    |  |  Signal    |
            |  Server    |==|  Server    |
            | (TCP:8743) |  | (TCP:8743) |
            +------------+  +------------+
                (same server instance)

     After signaling:

     +----+----+                     +----+----+
     | Peer A  |<====== UDP =======>| Peer B  |
     | (Client)|  Direct encrypted  | (Client)|
     +----+----+  P2P connection    +----+----+
```

### Internal Client Architecture

```
                          ┌─────────────┐
                          │   main.rs   │
                          │ (CLI parse) │
                          └──────┬──────┘
                                 │
                    ┌────────────┼────────────┐
                    │            │            │
              ┌─────┴─────┐ ┌───┴───┐ ┌─────┴─────┐
              │  commands/ │ │config │ │  identity  │
              │  connect   │ │peers  │ │  (keys)    │
              └─────┬──────┘ └───────┘ └────────────┘
                    │
       ┌────────────┼───────────────┐
       │            │               │
  ┌────┴────┐ ┌────┴────┐   ┌──────┴──────┐
  │  stun   │ │ signal  │   │ Terminal    │
  │ client  │ │ client  │   │ event loop  │
  └────┬────┘ └────┬────┘   └──────┬──────┘
       │           │               │
       └─────┬─────┘               │
             │                     │
       ┌─────┴─────┐        ┌─────┴─────┐
       │   punch   │        │    TUI     │
       │ (hole     │        │  (ratatui) │
       │  punch)   │        │            │
       └─────┬─────┘        │ ┌─────────┐│
             │               │ │ connect ││
       ┌─────┴─────┐        │ │  view   ││
       │ handshake │        │ ├─────────┤│
       │  (Noise)  │        │ │  chat   ││
       └─────┬─────┘        │ │  view   ││
             │               │ └─────────┘│
       ┌─────┴─────┐        └──────┬──────┘
       │  message  │◄──── AppEvent │
       │ send/recv │───── channel ─┘
       └───────────┘
```
