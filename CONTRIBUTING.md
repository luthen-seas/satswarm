# Contributing to Satswarm

Thank you for your interest in contributing to Satswarm!

## Getting Started

1. Fork the repository
2. Clone your fork: `git clone https://github.com/yourusername/satswarm`
3. Create a branch: `git checkout -b feature/your-feature`
4. Make your changes
5. Test: `cargo test`
6. Submit a pull request

## Development Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and build
git clone https://github.com/luthen-seas/satswarm
cd satswarm
cargo build
cargo test
```

## Code Standards

- Follow Rust formatting: `cargo fmt`
- Pass clippy lints: `cargo clippy`
- Add tests for new features
- Update documentation

## License

By contributing, you agree your contributions will be licensed under MIT.
