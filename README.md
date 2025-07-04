# mikriting-tool

A modern, modular monitoring and management tool for MikroTik routers with real-time VPN user monitoring, latency tracking, and network automation capabilities.

## Features

- 🔐 **Secure Authentication** - HTPassword-based authentication
- 📊 **Real-time Monitoring** - Live VPN user status and latency monitoring
- 🌐 **WebSocket Support** - Real-time updates without page refresh
- 🏗️ **Clean Architecture** - Modular, maintainable, and extensible design
- 🚀 **Fast Performance** - Built with Rust and Actix Web
- 📱 **Modern UI** - Responsive web interface
- 🔧 **Easy Configuration** - TOML-based configuration
- 📈 **Scalable Design** - Ready for enterprise deployment

## Architecture

This application follows Clean Architecture principles:

```
┌─────────────────────────────────────────────────────────────┐
│                    Presentation Layer                       │
│  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────┐  │
│  │   REST API      │  │   WebSocket     │  │  Static     │  │
│  │   (Actix Web)   │  │   (Real-time)   │  │  Files      │  │
│  └─────────────────┘  └─────────────────┘  └─────────────┘  │
└─────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────┐
│                    Use Cases Layer                          │
│  ┌─────────────────┐  ┌─────────────────┐                   │
│  │   VPN User      │  │   Auth          │                   │
│  │   Management    │  │   Management    │                   │
│  └─────────────────┘  └─────────────────┘                   │
└─────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────┐
│                    Domain Layer                             │
│  ┌─────────────────┐  ┌─────────────────┐                   │
│  │   Business      │  │   Domain        │                   │
│  │   Logic         │  │   Models        │                   │
│  └─────────────────┘  └─────────────────┘                   │
└─────────────────────────────────────────────────────────────┘
                                │
┌─────────────────────────────────────────────────────────────┐
│                Infrastructure Layer                         │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌─────────┐ │
│  │ MikroTik    │ │ Cache       │ │ Auth        │ │ Config  │ │
│  │ Client      │ │ Service     │ │ Service     │ │ Service │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └─────────┘ │
└─────────────────────────────────────────────────────────────┘
```

## Quick Start

### Prerequisites

- Rust 1.70+ installed
- MikroTik router with REST API enabled
- Network access to MikroTik router

### Installation

1. Clone the repository:
```bash
git clone <repository-url>
cd mikriting-tool
```

2. Copy configuration files:
```bash
cp config.toml_example config.toml
cp .htpasswd_example .htpasswd
```

3. Edit configuration:
```bash
nano config.toml
```

4. Build and run:
```bash
cargo build --release
./target/release/mikriting-tool
```

Or run in development mode:
```bash
cargo run
```

### Configuration

Edit `config.toml` to match your setup:

```toml
[app]
log_level = "info"
bind_address = "127.0.0.1"
bind_port = 3217
static_files_path = "./asset"
session_secret = "your-secret-key"
ping_interval_seconds = 2

[mikrotik]
protocol = "https"
address = "192.168.1.1"
port = 4343
username = "admin"
password = "your-password"
timeout_seconds = 10
```

### User Management

Create users in `.htpasswd` file:
```bash
htpasswd -B -c .htpasswd admin
```

## Usage

1. Start the application:
```bash
cargo run
```

2. Open your browser and go to `http://localhost:3217`

3. Login with your credentials

4. Monitor VPN users in real-time

## API Endpoints

- `GET /` - Main dashboard (requires authentication)
- `GET /login` - Login page
- `POST /login` - Login form submission
- `GET /logout` - Logout
- `GET /ws` - WebSocket connection for real-time updates
- `POST /api/trigger-update` - Manually trigger user list update
- `GET /api/users` - Get all VPN users (JSON)
- `POST /api/users/{username}/disconnect` - Disconnect specific user

## Development

### Project Structure

```
src/
├── domain/           # Business logic and models
│   ├── models.rs     # Domain models
│   ├── traits.rs     # Domain interfaces
│   └── mod.rs
├── usecase/          # Application use cases
│   ├── vpn_user.rs   # VPN user management
│   └── mod.rs
├── adapter/          # External interface adapters
│   ├── rest_api.rs   # HTTP REST API
│   ├── websocket.rs  # WebSocket handlers
│   ├── mikrotik/     # MikroTik API client
│   └── mod.rs
├── infrastructure/   # Infrastructure implementations
│   ├── cache.rs      # Caching service
│   ├── config.rs     # Configuration management
│   ├── auth.rs       # Authentication service
│   ├── ping.rs       # Ping monitoring
│   ├── repository.rs # Data repositories
│   ├── scheduler.rs  # Background tasks
│   └── mod.rs
└── main.rs          # Application entry point
```

### Adding New Features

1. **Add Domain Models**: Define new business entities in `domain/models.rs`
2. **Create Use Cases**: Implement business logic in `usecase/`
3. **Add Adapters**: Create external interfaces in `adapter/`
4. **Implement Infrastructure**: Add concrete implementations in `infrastructure/`
5. **Wire Dependencies**: Update `main.rs` for dependency injection

### Testing

Run tests:
```bash
cargo test
```

Run with coverage:
```bash
cargo tarpaulin --out html
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For support, please open an issue on GitHub or contact the maintainers.

## Roadmap

- [ ] Database persistence
- [ ] User management UI
- [ ] Advanced filtering and search
- [ ] Email notifications
- [ ] Metrics and analytics
- [ ] Multi-router support
- [ ] Docker deployment
- [ ] API documentation (OpenAPI)
- [ ] Rate limiting
- [ ] Audit logging
