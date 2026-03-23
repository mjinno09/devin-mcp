# Plan: Automate Cargo.toml Version on Release

## Goal

When a GitHub Release is published with a `vX.Y.Z` tag, automatically:
1. Validate tag format (`vX.Y.Z`)
2. Extract version from the tag and patch `Cargo.toml` in each job
3. Build release binaries with the correct version
4. Upload release assets to GitHub Release
5. Commit updated `Cargo.toml` and `Cargo.lock` back to `main`

## Changes

### `.github/workflows/release.yml`

- **Tag validation**: CI gate rejects tags not matching `vX.Y.Z`
- **Version sync**: Each job patches `Cargo.toml` in-place via `sed` before build
- **`update-version` job**: After release assets are uploaded, checks out `main`, updates `Cargo.toml` + `Cargo.lock`, and commits/pushes as `github-actions[bot]`

### Flow

```
Tag (vX.Y.Z) → Release published
  → CI Gate (validate tag + sync version + clippy/fmt)
  → Build (3 targets, version synced) → Upload assets
  → Commit version to main (Cargo.toml + Cargo.lock)
```
