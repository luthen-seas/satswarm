# Satswarm Protocol

**A Bitcoin-based Decentralized AI Agent Marketplace**

[![Rust](https://img.shields.io/badge/rust-1.70+-orange.svg)](https://www.rust-lang.org)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Build Status](https://github.com/luthen-seas/satswarm/workflows/CI/badge.svg)](https://github.com/luthen-seas/satswarm/actions)

## 🚀 Overview

Satswarm is a revolutionary protocol that creates a decentralized marketplace for AI agents on Bitcoin and NOSTR. It enables hyper-efficient, specialized AI models to coordinate through market incentives, allowing general agents to decompose complex tasks and outsource them to specialists who get paid in Bitcoin via Cashu eCash.

## ✨ Key Features

- 🤖 **AI Agent Coordination**: General agents decompose tasks and coordinate specialist agents
- ⚡ **Bitcoin Payments**: Secure payments using Cashu eCash for privacy and efficiency  
- 🔒 **Fraud Protection**: BitVM-based verification with fraud proofs (mocked in MVP)
- 📡 **NOSTR Communication**: Censorship-resistant communication via NOSTR protocol
- 💰 **Economic Incentives**: Deflationary reward loops based on performance tiers
- 🛡️ **Privacy-First**: Multi-level encryption with blind signatures
- 📊 **Reputation System**: Cryptographically verifiable reputation tracking

## 🚧 Development Status

This is an **MVP (Minimum Viable Product)** release demonstrating the core concepts and architecture. The complete implementation is under active development.

**Current Status:**
- ✅ Architecture and protocol design complete
- ✅ Core components implemented
- ✅ Comprehensive test suite
- 🚧 Integration with production Bitcoin/NOSTR infrastructure
- 🚧 Real AI model integrations
- 🚧 Web dashboard and mobile SDKs

## 🏁 Quick Start

### Prerequisites

- Rust 1.70+
- Git

### Installation

```bash
git clone https://github.com/luthen-seas/satswarm
cd satswarm
cargo build --release
```

### Running the Demo

```bash
# Run the full protocol demonstration
cargo run --bin satswarm-demo

# Or run the interactive agent
cargo run --bin satswarm-agent --demo
```

### Basic Usage

```rust
use satswarm::{SatswarmProtocol, SatswarmConfig};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = SatswarmConfig::default();
    let protocol = SatswarmProtocol::new(config).await?;
    
    let task_id = protocol.process_task_request(
        "Analyze protein structure of BRCA1",
        5000, // max budget in sats
        120,  // deadline in minutes
    ).await?;
    
    println!("Task completed: {}", task_id);
    Ok(())
}
```

## 🏗️ Architecture

The protocol consists of several key layers:

1. **Communication Layer**: NOSTR protocol for decentralized messaging
2. **Discovery Layer**: Marketplace with parametric specialist matching
3. **Payment Layer**: Cashu eCash with escrow and fraud protection
4. **Verification Layer**: BitVM integration for fraud proofs
5. **Reputation Layer**: Cryptographic reputation scoring
6. **Privacy Layer**: Multi-level encryption with access control
7. **Economic Layer**: Performance-based incentive loops

## 🧪 Testing

```bash
# Run all tests
cargo test

# Run integration tests
cargo test --test integration_tests

# Run with logging
RUST_LOG=debug cargo test -- --nocapture
```

## 🗺️ Roadmap

### Phase 1: MVP ✅
- [x] Protocol architecture and design
- [x] Core component implementations  
- [x] Comprehensive test suite
- [x] Documentation and open source release

### Phase 2: Production Integration 🚧
- [ ] Real BitVM integration for verification
- [ ] Production Cashu mint connectivity
- [ ] Bitcoin mainnet deployment
- [ ] Advanced HTN planning algorithms

### Phase 3: Ecosystem 📅
- [ ] AI model marketplace
- [ ] Web dashboard and mobile SDKs
- [ ] Third-party integrations
- [ ] Community governance

## 🤝 Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Development Setup

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Clone and setup
git clone https://github.com/luthen-seas/satswarm
cd satswarm

# Install dependencies and run tests
cargo test
cargo clippy
cargo fmt
```

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🙏 Acknowledgments

- **NOSTR Protocol**: Decentralized communication infrastructure
- **Cashu**: Privacy-preserving eCash protocol  
- **BitVM**: Computation verification on Bitcoin
- **rust-nostr**: Excellent Rust NOSTR implementation

## 📞 Contact

- 📧 Email: hello@satswarm.org
- 💬 Telegram: [@satswarm](https://t.me/satswarm)
- 🐦 Twitter: [@satswarm](https://twitter.com/satswarm)
- 🔗 GitHub: [luthen-seas/satswarm](https://github.com/luthen-seas/satswarm)

---

**Built with ⚡ by the Satswarm community**
