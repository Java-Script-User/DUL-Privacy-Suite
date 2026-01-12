# DUL Privacy Suite

[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/tauri-2.0-blue.svg)](https://tauri.app/)

A free, open-source privacy suite that protects your online activity through multi-layered security. Built for people who care about their digital privacy.

## Why DUL Privacy?

Traditional VPNs have limitations - they funnel all your traffic through a single company that can see everything you do. DUL Privacy takes a different approach by combining multiple privacy technologies into one easy-to-use application.

**What makes us different:**
- Multi-hop routing through Tor network - no single point sees your full traffic
- Built-in tracker blocking at the network level  
- WebRTC and IPv6 leak protection
- Kill switch prevents data leaks if connection drops
- Completely free and open source
- Modern, beautiful interface

## Download

**[Download for Windows](https://tbd/downloads)**

Choose your preferred version:
- **Installer (MSI)** - Recommended for most users, easy automatic updates
- **Portable (ZIP)** - No installation needed, run from anywhere

System Requirements: Windows 10/11 (64-bit)

## Features

## Quick Start

### For Users

1. Download and install DUL Privacy Suite
2. Run the application (requires Administrator)
3. Click the power button to connect
4. That's it! Your traffic is now protected

The app automatically configures your system proxy, so all applications benefit from the protection.

**Prerequisites:**
- Rust 1.70+
- Node.js 18+
- Windows 10/11

## Features in Detail

### Privacy Protection
- **Tor Network Integration**: Routes your traffic through the Tor network for anonymity
- **Tracker Blocking**: Blocks over 100+ known tracking domains at the network level
- **DNS Protection**: Prevents DNS leaks and uses encrypted DNS-over-HTTPS

### Security
- **Kill Switch**: Automatically blocks all traffic if the VPN connection drops
- **WebRTC Protection**: Prevents WebRTC leaks that can expose your real IP
- **IPv6 Protection**: Blocks IPv6 traffic to prevent IPv6 leaks

### Monitoring
- Real-time statistics dashboard
- View blocked trackers and requests
- Connection logs
- System status monitoring

### Core Modules

- **`proxy`**: Local HTTP/HTTPS proxy server
- **`routing`**: Multi-hop route selection and management
- **`crypto`**: Onion routing encryption layers
- **`network`**: Node registry and communication
- **`dns`**: Encrypted DNS resolution
- **`fingerprint`**: Browser fingerprint randomization
- **`blockchain`**: Payment and node rewards system

## Security Features

- **End-to-end encryption** for all traffic
- **Perfect forward secrecy** with ephemeral keys
- **No logging** of user activity
- **Open source** for full auditability
- **Decentralized** - no single point of failure

## Monetization (Coming Soon)

### For Users
- Pay only for what you use (per GB or per session)
- Cryptocurrency payments (ETH, BTC, others)
- No subscriptions, no personal information

### For Node Operators
- Earn cryptocurrency by running a node
- Automatic payments via smart contracts
- Stake-based reputation system

## Development

### Project Structure

```
privacy_suite/
├── src/
│   ├── main.rs           # Application entry point
│   ├── config.rs         # Configuration management
│   ├── proxy.rs          # HTTP/HTTPS proxy server
│   ├── routing.rs        # Multi-hop routing logic
│   ├── crypto.rs         # Encryption & onion routing
│   ├── network.rs        # Node registry & communication
│   ├── dns.rs            # Encrypted DNS resolver
│   ├── fingerprint.rs    # Browser fingerprint protection
│   └── blockchain.rs     # Blockchain payment integration
├── Cargo.toml            # Dependencies & project config
└── README.md             # This file
```

## Current Status

**DUL Privacy Suite v1.0** is a fully functional privacy application with:

- Modern desktop GUI built with Tauri
- Tor network integration for anonymous routing
- Real-time tracker blocking (100+ domains)
- WebRTC leak prevention
- IPv6 leak protection
- Kill switch for connection drops
- Automatic system proxy configuration
- Live statistics and monitoring dashboard
- Encrypted DNS-over-HTTPS
- Connection logging and filtering
- Multi-hop routing (in development)
- Blockchain payment integration (planned)

## Community & Support

- **Issues**: Report bugs on [GitHub Issues](https://github.com/java-script-user/dul-privacy-suite/issues)
- **Security**: For security vulnerabilities, email security@dulprivacy.com

Note: This is source-available software. The code is visible for transparency and security auditing, but not open for community contributions.

## Documentation

## Support the Project

If DUL Privacy Suite helps protect your privacy, consider supporting development:

- Star this repository
- Share on social media
- Donate cryptocurrency:
  - Bitcoin: `3CpHZqjxvQvXz64drZZxA6m3NWzKF1LCHX`
  - Ethereum: `0x3b867d70DD7C087374D05910F69C9BD2685b074D`

## License

DUL Privacy Suite is released under the [MIT License](LICENSE) for personal and non-commercial use.

### Enterprise & OEM Licensing

For commercial use, enterprise deployments, or OEM pre-installation:

- **Device Manufacturers**: License DUL Privacy for pre-installation on laptops, desktops, and mobile devices
- **Enterprise Organizations**: Custom deployments with dedicated support and SLA
- **Technology Companies**: White-label licensing and acquisition opportunities available

Interested in commercial licensing or acquisition? Contact: business@dulprivacy.com

## Important Notes

- **Privacy**: This software enhances your privacy but is not a guarantee of complete anonymity
- **Legal**: Use responsibly and in accordance with your local laws
- **Security**: While we take security seriously, no software is 100% secure
- **Open Source**: All code is open for review - we have nothing to hide

## Acknowledgments

Built with these amazing open-source projects:
- [Rust](https://www.rust-lang.org/) - Systems programming language
- [Tauri](https://tauri.app/) - Desktop application framework
- [Tor](https://www.torproject.org/) - Anonymity network
- [React](https://react.dev/) - UI framework

---

**Made with care for the privacy community**

*"Privacy is not something you should have to pay for. It's a fundamental right."*
