# Plan: Automate Cargo.toml Version on Release

## Goal

When a GitHub Release is published with a `vX.Y.Z` tag, automatically:
1. Extract the version from the tag
2. Update `Cargo.toml` version field
3. Build release binaries with the correct version
4. Publish to crates.io via `cargo publish`

## Changes

### `.github/workflows/release.yml`

- Add a `sync-version` step at the start of the CI gate job to validate the tag format
- In each build matrix job, inject `sed` to update `Cargo.toml` before `cargo build`
- Add a new `publish` job that:
  - Extracts version from tag
  - Updates `Cargo.toml`
  - Runs `cargo publish`
  - Requires `CARGO_REGISTRY_TOKEN` secret

### Why not commit the version change?

Committing during CI creates merge noise and circular triggers.
Instead, we patch `Cargo.toml` in-place during the workflow run only.
The source-of-truth for the version is the git tag.
