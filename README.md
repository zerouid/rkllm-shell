# RKLLM Shell

A Rust-based shell and HTTP server for interacting with RKLLM (Rockchip Large Language Model) runtime. This project provides both a command-line interface and a REST API server for running and managing LLM models on Rockchip hardware.

## Features

- **HTTP Server**: REST API compatible with OpenAI-style endpoints
- **Model Management**: Load, unload, and manage LLM models
- **Chat Completions**: Generate conversational responses
- **Text Completions**: Generate text completions
- **Embeddings**: Generate text embeddings
- **Configuration Management**: YAML-based configuration with sensible defaults
- **Logging**: Configurable logging levels for debugging and monitoring
- **Graceful Shutdown**: Proper cleanup on server termination

## Architecture

The project is structured as a Rust workspace with two main components:

- **`rkllm-api-sys`**: Low-level FFI bindings to the RKLLM C library
- **`rkllm-shell`**: High-level CLI application and HTTP server

## Installation

### Prerequisites

- Rust 1.70+ (2021 edition)
- RKLLM runtime library (`librkllmrt.so`)
- Rockchip hardware with NPU support

### Building from Source

```bash
git clone <repository-url>
cd rkllm-shell
cargo build --release
```

The compiled binary will be available at `target/release/rkllm-shell`.

## Usage

### Command Line Interface

```bash
# Show application information
rkllm-shell info

# Start the HTTP server
rkllm-shell serve

# Specify custom config directory
rkllm-shell -c /path/to/config serve

# Enable verbose logging
rkllm-shell -vvv serve
```

### Configuration

The application uses a YAML configuration file located in the user's config directory. On first run, a default configuration is created:

```yaml
models_path: "./data"
```

#### Configuration Options

- `models_path`: Directory where model files are stored (default: `"./data"`)

### HTTP API

The server provides OpenAI-compatible REST endpoints:

#### Chat Completions

```http
POST /api/chat
Content-Type: application/json

{
  "model": "model-name",
  "messages": [
    {
      "role": "user",
      "content": "Hello, how are you?"
    }
  ]
}
```

#### Text Completions

```http
POST /api/generate
Content-Type: application/json

{
  "model": "model-name",
  "prompt": "Complete this text:",
  "max_tokens": 100
}
```

#### Embeddings

```http
POST /api/embed
Content-Type: application/json

{
  "model": "model-name",
  "input": "Text to embed"
}
```

#### Model Management

```http
# List available models
GET /api/tags

# Show model information
POST /api/show
Content-Type: application/json

{
  "name": "model-name"
}

# Delete a model
DELETE /api/delete
Content-Type: application/json

{
  "name": "model-name"
}

# List running models
GET /api/ps

# Pull/download a model
POST /api/pull
Content-Type: application/json

{
  "name": "model-name"
}
```

#### Health Check

```http
GET /health
```

### Server Details

- **Port**: 3000 (default)
- **Host**: 0.0.0.0 (binds to all interfaces)
- **Graceful Shutdown**: Supports SIGTERM and Ctrl+C

## Development

### Project Structure

```
rkllm-shell/
├── rkllm-api-sys/          # FFI bindings to RKLLM C library
│   ├── build.rs            # Build script for generating bindings
│   ├── src/lib.rs           # Rust FFI interface
│   ├── vendor/              # RKLLM C library and headers
│   └── wrapper.hpp          # C++ wrapper for bindgen
├── rkllm-shell/            # Main application
│   ├── src/
│   │   ├── main.rs          # Application entry point
│   │   ├── args.rs          # Command-line argument parsing
│   │   ├── config/          # Configuration management
│   │   ├── commands/        # CLI commands (serve, info)
│   │   ├── server/          # HTTP server and API handlers
│   │   └── terminal/        # Terminal output formatting
│   └── Cargo.toml
├── docs/
│   └── tests.http          # HTTP API test examples
└── README.md
```

### Dependencies

Key dependencies include:

- **Web Framework**: Axum for HTTP server
- **CLI**: Clap for command-line parsing
- **Async Runtime**: Tokio for async operations
- **Serialization**: Serde for JSON handling
- **Configuration**: Config crate for YAML parsing
- **Logging**: Log crate with configurable levels
- **Terminal Colors**: Owo-colors for colored output

### Building FFI Bindings

The `rkllm-api-sys` crate uses `bindgen` to automatically generate Rust bindings from the C header files. The build process:

1. Downloads required dependencies if missing
2. Generates bindings from `rkllm.h`
3. Links against `librkllmrt.so`

### Testing

Use the provided HTTP test file for API testing:

```bash
# Install HTTPie or use curl
http POST localhost:3000/api/chat Content-Type:application/json model=test messages:='[{"role":"user","content":"Hello"}]'
```

## License

[License information to be added]

## Contributing

[Contributing guidelines to be added]

## Support

For issues and questions:

- Create an issue on the project repository
- Check the documentation in the `docs/` directory
- Review the HTTP API examples in `docs/tests.http`

## Changelog

### v0.1.0
- Initial release
- Basic HTTP server with OpenAI-compatible API
- Model management endpoints
- Configuration system
- CLI interface with info and serve commands
