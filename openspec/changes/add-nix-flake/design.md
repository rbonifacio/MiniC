## Context

MiniC is a Rust project. Developers need cargo, rustc, and common tooling (rustfmt, clippy, rust-analyzer). A Nix flake provides a reproducible dev shell via `nix develop`.

## Goals / Non-Goals

**Goals:**

- `nix develop` provides a shell with cargo, rustc, rustfmt, clippy, rust-analyzer
- Support macOS (darwin) and Linux (aarch64, x86_64)
- Use a recent, stable Rust toolchain

**Non-Goals:**

- Full Nix build of the crate (cargo build remains the primary build)
- CI integration (can be a follow-up)

## Decisions

### 1. Rust toolchain source

**Choice:** Use [oxalica/rust-overlay](https://github.com/oxalica/rust-overlay) for binary-distributed Rust toolchains.

**Rationale:** Provides up-to-date stable Rust; nixpkgs rust can be outdated. oxalica/rust-overlay is widely used and well-maintained.

### 2. Toolchain version

**Choice:** `rust-bin.stable.latest.default` — latest stable with default profile (rustc, cargo, rustfmt, clippy, rust-std).

**Rationale:** MiniC uses edition 2021; stable is sufficient. `latest` keeps the toolchain current; can pin later with `rust-toolchain.toml` if needed.

### 3. rust-analyzer

**Choice:** Include `rust-analyzer` from nixpkgs in the dev shell.

**Rationale:** Essential for IDE support; separate from the rust-overlay toolchain.

### 4. Multi-system support

**Choice:** Use `flake-utils` for `eachDefaultSystem` (linux x86_64/aarch64, darwin x86_64/aarch64).

**Rationale:** MiniC may be developed on macOS or Linux; flake-utils is the standard pattern.

### 5. Optional inputs

**Choice:** No pkg-config or openssl in dev shell unless needed. MiniC uses nom; no system C deps in Cargo.toml.

**Rationale:** Keep the dev shell minimal. Add if needed later.
