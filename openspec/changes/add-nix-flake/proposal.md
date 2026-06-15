## Why

Developers need a reproducible development environment for MiniC. A Nix flake provides `nix develop` to get a shell with the correct Rust toolchain (cargo, rustc, rustfmt, clippy), rust-analyzer, and any other build dependencies—without installing rustup or managing toolchains manually.

## What Changes

- **New**: `flake.nix` at project root
- **Optional**: `rust-toolchain.toml` or `rust-toolchain` to pin Rust version (can be inferred from Cargo.toml edition)
- **Optional**: `.envrc` with `use flake` for direnv users

## Capabilities

### New Capabilities

- `nix-develop`: `nix develop` yields a shell with cargo, rustc, rustfmt, clippy, rust-analyzer
- Reproducible: Same toolchain for all developers and CI

### Modified Capabilities

- None (additive)

## Impact

- **New**: `flake.nix`
- **Optional**: `rust-toolchain.toml`, `.envrc`
- **Breaking**: None
