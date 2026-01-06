# CLAUDE.md - AI Assistant Guide for hyveOS

## Project Overview

hyveOS is a decentralized robot communication system built by P2P Industries. It enables peer-to-peer communication between robots and IoT devices using a mesh network architecture based on B.A.T.M.A.N.-adv (Better Approach To Mobile Adhoc Networking - advanced).

**Documentation**: https://docs.p2p.industries

### Core Capabilities
- **Pub-Sub**: Topic-based publish/subscribe messaging via Gossipsub
- **Request-Response**: Direct peer-to-peer request/response communication
- **DHT (Key-Value Store)**: Distributed hash table for global data storage via Kademlia
- **Discovery**: Service discovery using DHT provider records
- **File Transfer**: P2P file sharing with CID-based content addressing
- **Local KV**: Persistent local key-value storage per node
- **App Deployment**: Docker-based application deployment to peers
- **Neighbours**: Direct peer connection monitoring

## Architecture Overview

hyveOS uses a layered actor-based architecture:

```
┌─────────────────────────────────────────────────────────────────────┐
│                        SDK Layer (User Applications)                │
│   Rust SDK | Python SDK | TypeScript SDK                           │
│   (gRPC clients over Unix socket or HTTP)                          │
└─────────────────────────────┬───────────────────────────────────────┘
                              │ gRPC (protobuf)
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        Bridge Layer (crates/bridge)                 │
│   gRPC Service Implementations                                      │
│   ReqResp | PubSub | KV | Discovery | FileTransfer | Apps | ...    │
└─────────────────────────────┬───────────────────────────────────────┘
                              │ Rust async channels (mpsc/oneshot)
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      P2P-Stack Layer (crates/p2p-stack)             │
│   Actor Model with Subactors                                        │
│   Client -> Commands -> Actor -> Subactors -> libp2p Behaviours    │
└─────────────────────────────┬───────────────────────────────────────┘
                              │ libp2p protocols
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        Network Layer (libp2p)                       │
│   Kademlia | Gossipsub | Request-Response | Identify | mDNS        │
│   BATMAN-adv | File Transfer Streams | Ping                        │
└─────────────────────────────────────────────────────────────────────┘
```

## Request Flow: SDK to Network

### Complete Request Flow Example

When an SDK calls `req_resp_service.send_request(peer_id, data)`:

```
1. SDK Layer (sdks/rust/src/services/req_resp.rs)
   └─ ReqRespService::send_request(peer_id, data, topic)
      └─ gRPC call: ReqRespClient::send(SendRequest)

2. Bridge Layer (crates/bridge/src/req_resp.rs)
   └─ ReqRespServer::send(request) receives gRPC call
      └─ self.client.req_resp().send_request(peer_id, msg)
         (calls p2p-stack Client)

3. P2P-Stack Client (crates/p2p-stack/src/subactors/req_resp.rs)
   └─ Creates oneshot channel for response
   └─ Sends Command::ReqResp(Request { peer_id, req, sender })
      via mpsc channel to actor

4. Actor Main Loop (crates/p2p-stack/src/actor.rs)
   └─ select! receives command from receiver.recv()
      └─ handle_command(Command::ReqResp(...))
         └─ self.req_resp.handle_command(cmd, behaviour)
            └─ behaviour.req_resp.send_request(peer_id, req)
               (libp2p request-response protocol)
            └─ Stores oneshot sender in pending_responses map

5. Network Layer (libp2p)
   └─ Serializes and sends request to remote peer
   └─ Remote peer processes and sends response

6. Response Path (reverse flow)
   └─ libp2p receives response, emits SwarmEvent
   └─ Actor handles event in select! loop
   └─ req_resp.handle_event() finds pending oneshot sender
   └─ sender.send(response) completes the oneshot
   └─ Bridge receives response, returns via gRPC
   └─ SDK receives response
```

### Connection Types

**Application Connection** (default for apps running in containers):
- SDK reads `HYVEOS_BRIDGE_SOCKET` environment variable
- Connects via Unix socket to bridge server
- Automatic heartbeat every 10 seconds

**CLI Connection** (for hyvectl):
- Connects to `/run/hyved/bridge/bridge.sock`
- No heartbeat required

**Network Connection** (with `network` feature):
- HTTP/gRPC connection to remote hyveOS runtime
- Supports IPv6 with zone IDs

## Repository Structure

```
hyveos/
├── crates/                    # Rust workspace crates
│   ├── hyved/                 # Main daemon entry point
│   ├── hyvectl/               # CLI tool
│   ├── hyvectl-commands/      # Shared CLI command definitions
│   ├── core/                  # Core types, gRPC codegen, protobuf
│   ├── config/                # Configuration parsing
│   ├── runtime/               # Runtime orchestration
│   ├── p2p-stack/             # libp2p actor and networking
│   ├── bridge/                # gRPC bridge server
│   ├── docker/                # Docker API integration
│   ├── batman-neighbours-core/  # BATMAN neighbor types
│   ├── batman-neighbours-daemon/ # BATMAN neighbor daemon
│   ├── ifaddr/                # Network interface utilities
│   ├── ifwatcher/             # Interface change monitoring
│   ├── macaddress/            # MAC address utilities
│   └── libp2p/                # Custom libp2p modules
│       ├── batman-adv/        # BATMAN-adv transport
│       └── addr-filter/       # Address filtering behaviour
├── sdks/                      # Client SDKs
│   ├── rust/                  # Rust SDK (hyveos-sdk)
│   ├── python/                # Python SDK (hyveos_sdk)
│   └── typescript/            # TypeScript/Deno SDKs
├── protos/bridge.proto        # gRPC API definition
├── ui/                        # Web UI (SvelteKit)
├── endpoint/data-collector/   # Data collection service
├── examples/                  # Example applications
└── demo/                      # Demo configurations
```

## Key Crates Explained

### crates/p2p-stack
The heart of the networking layer. Implements an actor model:

**Client** (`src/client.rs`): Handle for sending commands to the actor
```rust
pub struct Client {
    peer_id: PeerId,
    sender: mpsc::Sender<Command>,
}
// Provides typed accessors: .kad(), .gossipsub(), .req_resp(), .apps(), etc.
```

**Actor** (`src/actor.rs`): Main event loop processing commands and swarm events
```rust
// Runs two concurrent select branches:
// 1. Swarm events from libp2p (network activity)
// 2. Commands from Client (SDK requests)
```

**Subactors** (`src/subactors/`): Protocol-specific handlers
- `kad.rs` - Kademlia DHT operations
- `gossipsub.rs` - Pub-sub messaging
- `req_resp.rs` - Request-response protocol
- `apps.rs` - Application deployment protocol
- `file_transfer.rs` - File sharing
- `neighbours.rs` - Neighbor tracking

**MyBehaviour** (`src/behaviour.rs`): Combined libp2p NetworkBehaviour
```rust
#[derive(NetworkBehaviour)]
pub struct MyBehaviour {
    pub identify: identify::Behaviour,
    pub kad: kad::Behaviour,           // DHT
    pub gossipsub: gossipsub::Behaviour, // Pub-sub
    pub req_resp: req_resp::Behaviour,   // Request-response
    pub mdns: mdns::Behaviour,           // Local discovery
    pub ping: ping::Behaviour,
    pub apps: apps::Behaviour,           // Custom app protocol
    pub file_transfer: libp2p_stream::Behaviour,
    pub batman_neighbours: batman_adv::Behaviour,
}
```

### crates/bridge
gRPC server that translates SDK requests to p2p-stack commands:

```rust
pub struct BridgeClient<Db, Apps> {
    client: Client,           // p2p-stack client
    db_client: Db,           // Local database
    apps_client: Apps,       // Application manager
    base_path: PathBuf,      // Shared file directory
    // ...
}
```

Each gRPC service (ReqResp, PubSub, KV, etc.) delegates to the appropriate `client` method.

### crates/runtime
Orchestrates all components at startup:

```rust
pub struct Runtime {
    clients: Clients,
    actor_task: JoinHandle<()>,              // p2p-stack actor
    file_provider_task: JoinHandle<()>,       // File publishing
    application_manager_task: JoinHandle<()>, // Docker management
    ping_task: JoinHandle<()>,                // Health checks
    cli_bridge_task: JoinHandle<...>,         // gRPC server
}
```

### crates/core
Shared types and generated protobuf code:
- `req_resp.rs` - Request/Response types
- `pub_sub.rs` - Message types
- `file_transfer.rs` - Cid, FilePath
- `dht.rs` - DHT Key type
- `grpc/` - Generated protobuf bindings

## SDK Service Modules (Detailed)

### 1. Request-Response Service
**File**: `sdks/rust/src/services/req_resp.rs`

Peer-to-peer request-response with optional topic-based routing.

**Methods**:
- `send_request(peer_id, data, topic)` → `Response`
- `recv(query)` → `Stream<InboundRequest>`
- `respond(data)` / `respond_with_error(error)`

**Types**:
```rust
struct Request { data: Vec<u8>, topic: Option<String> }
enum Response { Data(Vec<u8>), Error(ResponseError) }
enum ResponseError { Timeout, TopicNotSubscribed, App(String) }
struct InboundRequest<T> { peer_id: PeerId, topic: Option<String>, data: T }
```

**Variants**: `JsonService<Req, Resp>`, `CborService<Req, Resp>` for typed serialization.

### 2. Pub-Sub Service (GossipSub)
**File**: `sdks/rust/src/services/pub_sub.rs`

Topic-based publish-subscribe using gossip protocol.

**Methods**:
- `publish(topic, data)` → `MessageId`
- `subscribe(topic)` → `Stream<ReceivedMessage>`
- `publish_json/cbor()`, `subscribe_json/cbor()` for typed messages

**Types**:
```rust
struct MessageId(Vec<u8>);
struct Message { data: Vec<u8>, topic: String }
struct ReceivedMessage {
    propagation_source: PeerId,  // Direct sender
    source: Option<PeerId>,      // Original publisher
    message_id: MessageId,
    message: Message,
}
```

### 3. DHT Key-Value Store
**File**: `sdks/rust/src/services/kv.rs`

Distributed hash table for network-wide key-value storage.

**Methods**:
- `put_record(topic, key, value)` → `()`
- `get_record(topic, key)` → `Option<Vec<u8>>`
- `remove_record(topic, key)` → `()` (local effect only)
- Typed variants: `put_record_json/cbor()`, `get_record_json/cbor()`

**Types**:
```rust
struct Key { topic: String, key: Vec<u8> }
// Encoded internally as "topic/key" - topic cannot contain '/'
```

### 4. Local Key-Value Store
**File**: `sdks/rust/src/services/local_kv.rs`

Persistent local storage (not shared across network).

**Methods**:
- `put(key, value)` → `Option<Vec<u8>>` (returns previous value)
- `get(key)` → `Option<Vec<u8>>`
- Typed variants available

### 5. Discovery Service
**File**: `sdks/rust/src/services/discovery.rs`

Service discovery using DHT provider records.

**Methods**:
- `provide(topic, key)` → `()` (register as provider)
- `get_providers(topic, key)` → `Stream<PeerId>`
- `stop_providing(topic, key)` → `()` (local effect only)

### 6. File Transfer Service
**File**: `sdks/rust/src/services/file_transfer.rs`

Content-addressed file distribution.

**Methods**:
- `publish(path)` → `Cid`
- `get(cid)` → `PathBuf`
- `get_with_progress(cid)` → `Stream<DownloadEvent>`

**Types**:
```rust
struct Cid {
    id: Ulid,           // Unique identifier
    hash: [u8; 32],     // SHA-256 content hash
}
// Format: "ULID-hexhash"

enum DownloadEvent {
    Progress(u64),      // Percentage 0-100
    Ready(PathBuf),     // Complete file path
}
```

### 7. Neighbours Service
**File**: `sdks/rust/src/services/neighbours.rs`

Direct peer connection monitoring.

**Methods**:
- `subscribe()` → `Stream<NeighbourEvent>`
- `get()` → `Vec<PeerId>`

**Types**:
```rust
enum NeighbourEvent {
    Init(Vec<PeerId>),    // Initial list (always first)
    Discovered(PeerId),   // New connection
    Lost(PeerId),         // Connection lost
}
```

### 8. Debug Service
**File**: `sdks/rust/src/services/debug.rs`

Network monitoring and debugging.

**Methods**:
- `subscribe_mesh_topology()` → `Stream<MeshTopologyEvent>`
- `subscribe_messages()` → `Stream<MessageDebugEvent>`

**Types**:
```rust
struct MeshTopologyEvent { peer_id: PeerId, event: NeighbourEvent }
struct MessageDebugEvent { sender: PeerId, event: MessageDebugEventType }
enum MessageDebugEventType {
    Request(RequestDebugEvent),
    Response(ResponseDebugEvent),
    PubSub(Message),
}
```

### 9. Apps Service
**File**: `sdks/rust/src/services/apps.rs`

Docker application management across the mesh.

**Methods**:
- `deploy(config)` → `Ulid` (app ID)
- `list_running(target_peer_id)` → `Vec<RunningApp>`
- `stop(id, target_peer_id)` → `()`
- `get_own_app_id()` → `Ulid`

**Types**:
```rust
struct Config {
    image: String,                    // Docker image
    local: bool,                      // Image available locally
    target_peer_id: Option<PeerId>,   // Deploy target (None = local)
    exposed_ports: Option<Vec<u16>>,
    persistent: bool,                 // Restart on daemon restart
}

struct RunningApp { id: Ulid, image: String }
```

### 10. Connection/Control
**File**: `sdks/rust/src/connection.rs`

Connection management and identity.

**Methods**:
- `Connection::new()` → Default connection (reads `HYVEOS_BRIDGE_SOCKET`)
- `Connection::builder()` → `ConnectionBuilder`
- `connection.get_id()` → `PeerId`

**Builder Options**:
- `.custom(socket_path, shared_dir)` - Custom Unix socket
- `.uri(uri)` - Network connection (requires `network` feature)
- `.heartbeat_interval(duration)` - Custom heartbeat (default 10s)

## Protocol Buffers (protos/bridge.proto)

Defines 10 gRPC services:

| Service | RPCs | Purpose |
|---------|------|---------|
| ReqResp | Send, Recv (stream), Respond | Request-response messaging |
| Neighbours | Subscribe (stream), Get | Peer connection events |
| PubSub | Subscribe (stream), Publish | Topic-based messaging |
| KV | PutRecord, GetRecord, RemoveRecord | Distributed key-value |
| Discovery | Provide, GetProviders (stream), StopProviding | Service discovery |
| LocalKV | Put, Get | Local persistent storage |
| FileTransfer | Publish, Get, GetWithProgress (stream) | File sharing |
| Debug | SubscribeMeshTopology (stream), SubscribeMessages (stream) | Monitoring |
| Apps | Deploy, ListRunning, Stop, GetOwnAppId | App management |
| Control | Heartbeat, GetId | Runtime control |

## Development Setup

### Prerequisites
- **Rust**: 1.80.0+ (specified in `Cargo.toml`)
- **protoc**: Protocol Buffer compiler
  ```bash
  # macOS
  brew install protobuf
  # Ubuntu/Debian
  sudo apt install protobuf-compiler
  ```
- **Docker**: For container-based features
- **Nix** (optional): `nix develop` for complete dev environment

### Build Commands

```bash
# Build all default members
cargo build

# Build specific crate
cargo build -p hyved
cargo build -p hyvectl
cargo build -p hyveos-sdk

# Build with all features
cargo build --all-features

# Build with specific features
cargo build -p hyved -F batman
cargo build -p hyved -F batman,network

# Release build
cargo build --release

# Performance profile (LTO + single codegen unit)
cargo build --profile perf

# Size-optimized profile
cargo build --profile size
```

### Test Commands

```bash
# Run all tests
cargo test

# Test specific crate
cargo test -p hyveos-sdk

# Test with all features
cargo test --all-features

# Test without default features
cargo test --no-default-features
```

### Linting and Formatting

```bash
# Format code
cargo fmt

# Check formatting
cargo fmt -- --check

# Run clippy
cargo clippy --all-features -- -D warnings
cargo clippy --no-default-features -- -D warnings

# Check dependencies (licenses, advisories)
cargo deny check advisories bans licenses sources
```

## Code Style Conventions

### Rust
Configuration in `.rustfmt.toml`:
- Max width: 100 characters
- Import grouping: `StdExternalCrate` (std first, then external, then crate)
- Import granularity: `Crate`
- Macro formatting enabled

Additional conventions:
- Use `hyveOS` in documentation (configured in `clippy.toml`)
- GPL-3.0-only license for workspace crates
- MIT license for SDK crate (`hyveos-sdk`)
- All async code uses Tokio runtime
- Tracing instrumentation on public methods

### Python SDK
- Python 3.11+
- Uses `ruff` for formatting and linting
- Single quotes, 4-space indentation
- Check: `poetry run ruff format --check && poetry run ruff check`

### TypeScript SDK
- Deno 2.x
- Standard Deno formatting and linting
- Check: `deno check *.ts && deno lint && deno fmt --check`

### UI (SvelteKit)
- Node.js 20, pnpm 9
- Svelte 5 with TypeScript
- TailwindCSS + DaisyUI
- Check: `pnpm check && pnpm lint`

## CI/CD Pipeline

### Rust Workflow (`.github/workflows/rust.yml`)
Triggers: push to main, PRs, tags `v*.*.*-*-rust`

Jobs:
1. **test**: Build and test each crate on x86_64 and aarch64
2. **clippy**: Lint with/without all features (`-D warnings`)
3. **rustfmt**: Check formatting
4. **ensure-lockfile-uptodate**: Verify Cargo.lock consistency
5. **cargo-deny**: Check licenses and security advisories
6. **publish**: Publish to crates.io on rust tags

### Python Workflow (`.github/workflows/python.yml`)
Triggers: push to main, PRs, tags `v*-python`

Jobs:
1. **build**: Format check, lint, verify protobuf, build SDK
2. **build-doc**: Generate pdoc documentation
3. **publish**: Publish to PyPI on python tags

### Frontend Workflow (`.github/workflows/frontend.yml`)
Triggers: push to main, PRs

Jobs:
1. **build-deno**: Build TypeScript packages
2. **test-deno**: Check, lint, format TypeScript
3. **check**: Build and lint SvelteKit UI

## Concurrency Model

```
Runtime Process
├─ Actor Task (p2p-stack)
│  ├─ Swarm event loop (libp2p networking)
│  └─ Command processing (from Clients)
│
├─ Application Manager Task
│  ├─ Command broker (apps behavior messages)
│  ├─ Docker container lifecycle
│  └─ Per-app bridge management
│
├─ CLI Bridge Task
│  ├─ gRPC server for hyvectl
│  └─ Request routing to p2p-stack
│
├─ File Provider Task
│  └─ File transfer handling
│
└─ Ping Task
   └─ Periodic health checks
```

All tasks communicate via:
- `mpsc` channels for commands
- `oneshot` channels for responses
- `broadcast` channels for events

## Cross-Compilation

```bash
# Install cross-compile toolchain (Ubuntu)
sudo apt install gcc-aarch64-linux-gnu

# Build for ARM64
cargo build --target aarch64-unknown-linux-gnu
```

Linker configuration in `.cargo/config.toml`.

## Debian Packaging

```bash
# Build .deb packages
cargo deb -p hyved
cargo deb -p hyvectl
```

## Common Development Tasks

### Adding a new gRPC service
1. Define messages and service in `protos/bridge.proto`
2. Regenerate Rust bindings: `cargo build -p hyveos-core`
3. Regenerate Python bindings: `cd sdks/python && ./generate.sh ./hyveos_sdk/protocol`
4. Implement server in `crates/bridge/src/{service}.rs`
5. Add subactor in `crates/p2p-stack/src/subactors/` if needed
6. Add client methods to SDKs

### Adding a new Rust crate
1. Create crate in `crates/`
2. Add to `workspace.members` in root `Cargo.toml`
3. Use `workspace = true` for shared dependencies
4. Add to `default-members` if it should build by default

### Tracing a bug through the stack
1. Start at SDK service method
2. Find corresponding Bridge gRPC handler
3. Trace to p2p-stack Client method
4. Find Command enum variant
5. Find Subactor command handler
6. Check libp2p Behaviour interaction

## Important Files

| File | Purpose |
|------|---------|
| `Cargo.toml` | Workspace configuration, shared dependencies |
| `deny.toml` | Dependency license and security policy |
| `.rustfmt.toml` | Rust formatting rules |
| `clippy.toml` | Clippy configuration |
| `protos/bridge.proto` | gRPC API definition |
| `flake.nix` | Nix development environment |
| `.cargo/config.toml` | Cross-compilation linker config |

## External Dependencies

Notable dependencies:
- **libp2p**: Custom fork at `github.com/p2p-industries/rust-libp2p`
- **tonic/prost**: gRPC implementation and protobuf
- **tokio**: Async runtime
- **tarpc**: Internal RPC (apps protocol)

## Versioning

- Rust crates: Tag `v{version}-{crate-name}-rust`
- Python SDK: Tag `v{version}-python`
- Minimum Rust: 1.80.0

## Contact

Maintainers:
- Hannes Furmans (hannes@p2p.industries)
- Josef Zoller (josef@p2p.industries)
- Linus Mierhoefer (linus@p2p.industries)
- Lukas Ego (lukas@p2p.industries)
