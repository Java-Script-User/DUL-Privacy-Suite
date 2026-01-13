# DUL Privacy Suite

[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![Tauri](https://img.shields.io/badge/tauri-2.0-blue.svg)](https://tauri.app/)

A privacy-focused desktop application that routes your traffic through the Tor network with built-in tracker blocking and leak protection.

## Why DUL Privacy?

Traditional VPNs route all your traffic through a single company that can see everything you do. DUL Privacy uses the Tor network instead, which routes your traffic through multiple volunteer-operated servers around the world. No single server sees both where you're connecting from and where you're going.

**Key Features:**
- Routes traffic through Tor network (automatic 3-hop routing)
- Blocks tracking domains at the network level  
- Prevents WebRTC and IPv6 leaks
- Kill switch blocks traffic if connection drops
- Free for personal use
- Clean, modern interface

## Download

Download for Windows from the releases page.

**Available versions:**
- Installer (MSI) - Recommended, includes automatic updates
- Portable (ZIP) - Run without installation

System Requirements: Windows 10/11 (64-bit)

## Quick Start

1. Download and install DUL Privacy Suite
2. Run as Administrator
3. Click the power button to connect
4. Your traffic is now routed through Tor

The application automatically configures your system proxy settings.

## Features

### Privacy & Anonymity
- **Tor Network Integration**: Traffic routed through 3 volunteer-operated servers (Guard, Middle, Exit nodes)
- **Multi-hop Routing**: No single server sees both your source and destination
- **Onion Encryption**: Multiple layers of encryption, peeled off at each hop
- **Tracker Blocking**: Blocks 99+ known tracking domains before requests are sent
- **DNS-over-HTTPS**: Encrypted DNS queries prevent ISP snooping

### Leak Protection
- **Kill Switch**: Blocks all network traffic if Tor connection fails
- **WebRTC Protection**: Prevents browser WebRTC from leaking your real IP address
- **IPv6 Protection**: Blocks IPv6 traffic to prevent IPv6 leaks

### Monitoring
- Real-time statistics dashboard
- Live tracker blocking logs
- Connection status monitoring
- Request filtering and search

## Technical Details

### Architecture

```
privacy_suite/
├── src/
│   ├── main.rs              # Application entry point
│   ├── proxy.rs             # HTTP/HTTPS proxy server
│   ├── tor_network.rs       # Tor network integration
│   ├── blocklist.rs         # Tracker domain blocking
│   ├── dns.rs               # DNS-over-HTTPS resolver
│   ├── fingerprint.rs       # Browser fingerprint randomization
│   ├── webrtc_protection.rs # WebRTC leak prevention
│   ├── ipv6_protection.rs   # IPv6 leak prevention
│   ├── kill_switch.rs       # Network kill switch
│   └── web_api.rs           # Statistics and logging API
├── gui/                     # Tauri desktop application
└── Cargo.toml
```

### How It Works

1. **Proxy Layer**: Local HTTP/HTTPS proxy intercepts browser traffic
2. **Filtering**: Checks requests against tracker blocklist and leak protection rules
3. **Tor Routing**: Allowed traffic is routed through Tor network (3 hops with onion encryption)
4. **Monitoring**: Statistics and logs are collected for the dashboard

### Built With

- **Rust** - Core proxy and privacy logic
- **Tauri** - Cross-platform desktop application framework
- **React** - User interface
- **Arti** - Rust implementation of Tor
- **Hickory DNS** - DNS-over-HTTPS client

## Status

Current version includes:

- Desktop GUI (Windows)
- Tor network integration
- Tracker blocking (99 domains)
- WebRTC leak prevention
- IPv6 leak protection
- Kill switch
- System proxy configuration
- DNS-over-HTTPS
- Live statistics dashboard
- Connection logging

## Support & Issues

Report bugs or request features through GitHub Issues.

For security vulnerabilities: security@dulprivacy.com

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
