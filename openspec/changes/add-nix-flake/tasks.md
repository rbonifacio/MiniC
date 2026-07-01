## 1. Create flake.nix

- [x] 1.1 Add inputs: nixpkgs, rust-overlay, flake-utils
- [x] 1.2 Add `devShells.default` with rust-overlay, rust-bin.stable.latest.default, rust-analyzer
- [x] 1.3 Use flake-utils eachDefaultSystem for multi-platform support

## 2. Optional: rust-toolchain

- [x] 2.1 Skip: using latest stable is fine for now

## 3. Optional: direnv

- [x] 3.1 Add `.envrc` with `use flake` for direnv users

## 4. Verify

- [ ] 4.1 Run `nix develop` and verify `cargo build`, `cargo test`, `rust-analyzer --version` work
