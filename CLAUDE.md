# CLAUDE.md - AI Assistant Guide for hyveOS

## Project Overview

hyveOS is a decentralized robot communication system built by P2P Industries. It enables peer-to-peer communication between robots and IoT devices using a mesh network architecture based on B.A.T.M.A.N.-adv (Better Approach To Mobile Adhoc Networking - advanced).

**Documentation**: https://docs.p2p.industries

### Core Capabilities
- **Pub-Sub**: Topic-based publish/subscribe messaging
- **Request-Response**: Direct peer-to-peer request/response communication
- **DHT (Key-Value Store)**: Distributed hash table for global data storage
- **Discovery**: Service discovery using provider records
- **File Transfer**: P2P file sharing with CID-based addressing
- **App Deployment**: Docker-based application deployment to peers

## Repository Structure

```
hyveos/
├── crates/                    # Rust workspace crates
│   ├── hyved/                 # Main hyveOS daemon
│   ├── hyvectl/               # CLI tool for controlling hyved
│   ├── hyvectl-commands/      # Shared command definitions
│   ├── core/                  # Core types and gRPC definitions
│   ├── config/                # Configuration management
│   ├── runtime/               # Runtime implementation
│   ├── p2p-stack/             # libp2p networking stack
│   ├── bridge/                # gRPC bridge server
│   ├── docker/                # Docker integration
│   ├── batman-neighbours-core/  # B.A.T.M.A.N. neighbor discovery (core)
│   ├── batman-neighbours-daemon/ # B.A.T.M.A.N. neighbor discovery (daemon)
│   ├── ifaddr/                # Network interface address utilities
│   ├── ifwatcher/             # Network interface watcher
│   ├── macaddress/            # MAC address utilities
│   └── libp2p/                # Custom libp2p modules
│       ├── batman-adv/        # B.A.T.M.A.N.-adv transport
│       └── addr-filter/       # Address filtering
├── sdks/                      # Client SDKs
│   ├── rust/                  # Rust SDK (hyveos-sdk)
│   ├── python/                # Python SDK (hyveos_sdk)
│   └── typescript/            # TypeScript/Deno SDKs
│       ├── hyveos-sdk/        # Core SDK
│       ├── hyveos-web/        # Browser SDK
│       └── hyveos-server/     # Server SDK
├── protos/                    # Protocol Buffer definitions
│   └── bridge.proto           # gRPC service definitions
├── ui/                        # Web UI (SvelteKit + Tailwind)
├── endpoint/                  # Endpoint services
│   └── data-collector/        # Data collection service
├── examples/                  # Example applications
└── demo/                      # Demo configurations
```

## Key Components

### hyved (Daemon)
The main hyveOS daemon that runs on each node. Features:
- `batman`: B.A.T.M.A.N.-adv mesh networking support
- `mdns`: mDNS discovery
- `network`: Network interface management

### hyvectl (CLI)
Command-line tool for interacting with hyved:
- Query peer information (`hyvectl whoami`)
- Manage applications
- Interact with DHT, pub-sub, etc.

### SDKs
Multi-language SDKs communicate with hyved via gRPC:
- **Rust**: `hyveos-sdk` crate with features: `cbor`, `json`, `network`, `app-management`
- **Python**: `hyveos-sdk` package using grpcio
- **TypeScript**: Deno-based packages for browser and server

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

# Release build (optimized)
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

### Python SDK
Configuration in `pyproject.toml`:
- Python 3.11+
- Uses `ruff` for formatting and linting
- Single quotes, 4-space indentation
- Check with: `poetry run ruff format --check` and `poetry run ruff check`

### TypeScript SDK
- Uses Deno 2.x
- Standard Deno formatting and linting
- Check with: `deno check`, `deno lint`, `deno fmt --check`

### UI (SvelteKit)
- Node.js 20, pnpm 9
- Svelte 5 with TypeScript
- TailwindCSS + DaisyUI
- Prettier + ESLint for formatting/linting
- Check with: `pnpm check`, `pnpm lint`

## CI/CD Pipeline

### Rust Workflow (`.github/workflows/rust.yml`)
Runs on: push to main, PRs, version tags (`v*.*.*-*-rust`)

Jobs:
1. **test**: Build and test each crate on x86_64 and aarch64
2. **clippy**: Lint with/without all features
3. **rustfmt**: Check formatting
4. **ensure-lockfile-uptodate**: Verify Cargo.lock consistency
5. **cargo-deny**: Check licenses and security advisories
6. **publish**: Publish to crates.io on rust tags

### Python Workflow (`.github/workflows/python.yml`)
Runs on: push to main, PRs, version tags (`v*-python`)

Jobs:
1. **build**: Format check, lint, build SDK
2. **build-doc**: Generate pdoc documentation
3. **publish**: Publish to PyPI on python tags

### Frontend Workflow (`.github/workflows/frontend.yml`)
Runs on: push to main, PRs

Jobs:
1. **build-deno**: Build TypeScript packages (hyveos-sdk, hyveos-web, hyveos-server)
2. **test-deno**: Check, lint, format TypeScript packages
3. **check**: Build and lint SvelteKit UI

## Protocol Buffers

The gRPC API is defined in `protos/bridge.proto`. Services:
- **ReqResp**: Request-response messaging
- **Neighbours**: Neighbor discovery
- **PubSub**: Publish-subscribe messaging
- **KV**: Global DHT key-value store
- **Discovery**: Service discovery
- **LocalKV**: Local key-value store
- **FileTransfer**: File sharing
- **Debug**: Debugging/monitoring
- **Apps**: Docker app management
- **Control**: Runtime control (heartbeat, ID)

When modifying `.proto` files, regenerate bindings:
```bash
# Python
cd sdks/python && ./generate.sh ./hyveos_sdk/protocol
```

## Cross-Compilation

The project supports cross-compilation to aarch64 (ARM64):

```bash
# Install cross-compile toolchain (Ubuntu)
sudo apt install gcc-aarch64-linux-gnu

# Build for ARM64
cargo build --target aarch64-unknown-linux-gnu
```

Linker configuration in `.cargo/config.toml`.

## Debian Packaging

Both `hyved` and `hyvectl` include Debian package metadata:
```bash
# Build .deb packages
cargo deb -p hyved
cargo deb -p hyvectl
```

## Common Development Tasks

### Adding a new gRPC service
1. Define messages and service in `protos/bridge.proto`
2. Regenerate bindings for all SDKs
3. Implement server-side in `crates/bridge/`
4. Add client methods to SDKs

### Adding a new Rust crate
1. Create crate in `crates/` or appropriate location
2. Add to workspace members in root `Cargo.toml`
3. Use workspace dependencies where possible
4. Add crate name to `default-members` if it should build by default

### Testing SDK changes
```bash
# Rust SDK
cargo test -p hyveos-sdk --all-features

# Python SDK
cd sdks/python/hyveos_sdk
poetry install
poetry run ruff check
poetry run ruff format --check

# TypeScript SDK
cd sdks/typescript/hyveos-sdk
deno check *.ts
deno lint
deno fmt --check
```

## Important Files

- `Cargo.toml` - Workspace configuration and shared dependencies
- `deny.toml` - Dependency license and security policy
- `.rustfmt.toml` - Rust formatting rules
- `clippy.toml` - Clippy configuration
- `protos/bridge.proto` - gRPC API definition
- `flake.nix` - Nix development environment

## Versioning and Releases

- Rust crates: Tag format `v{version}-{crate-name}-rust` (e.g., `v0.1.0-hyveos-sdk-rust`)
- Python SDK: Tag format `v{version}-python` (e.g., `v0.1.1-python`)
- Minimum Rust version: 1.80.0

## External Dependencies

Notable external dependencies:
- **libp2p**: Custom fork at `github.com/p2p-industries/rust-libp2p`
- **tonic/prost**: gRPC implementation
- **tokio**: Async runtime
- **tarpc**: RPC framework (for internal use)

## Contact

Maintainers (alphabetical):
- Hannes Furmans (hannes@p2p.industries)
- Josef Zoller (josef@p2p.industries)
- Linus Mierhoefer (linus@p2p.industries)
- Lukas Ego (lukas@p2p.industries)
