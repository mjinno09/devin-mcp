# Contributing to devin-mcp

Thank you for your interest in contributing!

## Development Setup

```sh
git clone https://github.com/mjinno09/devin-mcp.git
cd devin-mcp
cargo build
```

## Running Tests

```sh
# Unit tests (no API key needed)
cargo test --bin devin-mcp

# Integration tests — mock only (no API key needed)
cargo test --test integration

# Integration tests — live (requires DEVIN_API_KEY)
DEVIN_API_KEY=your_key cargo test --test integration
```

## Code Quality

Please ensure all checks pass before submitting a PR:

```sh
cargo fmt --all --check
cargo clippy --all-targets --all-features
cargo test
```

## Pull Request Process

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Run `cargo fmt` and `cargo clippy`
5. Run `cargo test --bin devin-mcp` to ensure unit tests pass
6. Commit your changes with a descriptive message
7. Push to your fork and open a Pull Request

## Reporting Issues

- Use the Bug Report template for bugs
- Use the Feature Request template for new ideas
- Include reproduction steps and your environment info

## Code of Conduct

Be respectful, constructive, and collaborative.
