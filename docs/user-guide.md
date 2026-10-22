# RKLLM Shell User Guide

A comprehensive guide for using RKLLM Shell - a Rust-based CLI and HTTP server for running LLM models on Rockchip hardware.

## Table of Contents

1. [Quick Start](#quick-start)
2. [Installation](#installation)
3. [Configuration](#configuration)
4. [Command Line Interface](#command-line-interface)
5. [HTTP API Reference](#http-api-reference)
6. [Model Management](#model-management)
7. [Examples](#examples)
8. [Troubleshooting](#troubleshooting)
9. [Advanced Usage](#advanced-usage)

---

## Quick Start

```bash
# 1. Build the project
cargo build --release

# 2. Start the server (default port 3000)
./target/release/rkllm-shell serve

# 3. In another terminal, test the API
curl -X POST http://localhost:3000/api/chat \
  -H "Content-Type: application/json" \
  -d '{"model": "your-model", "messages": [{"role": "user", "content": "Hello!"}]}'
```

---

## Installation

### Prerequisites

- **Rust 1.70+** (2021 edition)
- **RKLLM runtime library** (`librkllmrt.so`)
- **Rockchip hardware** with NPU support (RK3588, RK3576, etc.)
- **Linux** (tested on Ubuntu 20.04+, Debian 11+)

### Building from Source

```bash
# Clone the repository
git clone https://github.com/zerouid/rkllm-shell.git
cd rkllm-shell

# Build in release mode
cargo build --release

# The binary will be at target/release/rkllm-shell
```

### Installing System-wide (Optional)

```bash
# Install to ~/.cargo/bin
cargo install --path rkllm-shell

# Or copy manually
cp target/release/rkllm-shell /usr/local/bin/
```

---

## Configuration

### Configuration File Location

The configuration file is automatically created on first run at:

| OS | Path |
|----|------|
| Linux | `~/.config/rkllm-shell/config.yaml` |
| macOS | `~/Library/Application Support/rkllm-shell/config.yaml` |

### Configuration Options

```yaml
# config.yaml
models_path: "./data"    # Directory containing model files (relative to config dir or absolute)
```

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `models_path` | string | `"./data"` | Path to directory containing RKLLM model files |

### Environment Variables

| Variable | Description |
|----------|-------------|
| `RKLLM_SHELL_CONFIG` | Override config file location |
| `RUST_LOG` | Set log level (e.g., `debug`, `info`, `warn`, `error`) |

---

## Command Line Interface

### Global Options

```bash
rkllm-shell [OPTIONS] <COMMAND>

Options:
  -c, --config <PATH>    Specify custom config directory
  -v, --verbose          Increase verbosity (use -v, -vv, -vvv)
  -q, --quiet            Suppress non-error output
  -h, --help             Print help
  -V, --version          Print version
```

### Commands

#### `info` - Show Application Information

Displays build info, configuration, and detected hardware.

```bash
rkllm-shell info

# With custom config
rkllm-shell -c /path/to/config info

# Verbose output
rkllm-shell -vv info
```

**Example output:**
```
RKLLM Shell v0.1.0
Build: release (2026-01-15)
RKLLM Runtime: 1.3.1
NPU Driver: 0.9.8
Platform: RK3588
Config: /home/user/.config/rkllm-shell/config.yaml
Models path: /home/user/.config/rkllm-shell/data
Available models: 3
  - llama-3.2-1b (Q4_K_M, 1.2 GB)
  - qwen-2.5-0.5b (Q4_K_M, 450 MB)
  - phi-3-mini (Q4_K_M, 2.1 GB)
```

#### `serve` - Start HTTP Server

Starts the REST API server on port 3000.

```bash
# Default (port 3000, all interfaces)
rkllm-shell serve

# Custom config directory
rkllm-shell -c /path/to/config serve

# With debug logging
rkllm-shell -vvv serve
```

**Server details:**
- **Port**: 3000
- **Host**: 0.0.0.0 (all interfaces)
- **Graceful shutdown**: SIGTERM, SIGINT (Ctrl+C)

---

## Model Management

### Model Directory Structure

Models should be placed in the configured `models_path` directory:

```
models/
├── llama-3.2-1b/
│   └── model.rkllm
├── qwen-2.5-0.5b/
│   └── model.rkllm
└── phi-3-mini/
    └── model.rkllm
```

### Supported Model Formats

- **RKLLM** (`.rkllm`) - Native Rockchip format
- Models converted from GGUF/HuggingFace using RKNPU toolkit

### Model Naming

Model names are derived from directory names. Use descriptive names:
- `llama-3.2-1b-q4km`
- `qwen2.5-0.5b-instruct`
- `phi-3-mini-4k`

---

## Examples

### Example 1: Basic Chat with curl

```bash
# Non-streaming chat
curl -X POST http://localhost:3000/api/chat \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama-3.2-1b",
    "messages": [
      {"role": "system", "content": "You are a concise assistant."},
      {"role": "user", "content": "What is the capital of France?"}
    ],
    "temperature": 0.7,
    "max_tokens": 50
  }'
```

### Example 2: Streaming Chat

```bash
# Streaming chat (Server-Sent Events)
curl -X POST http://localhost:3000/api/chat \
  -H "Content-Type: application/json" \
  -N \
  -d '{
    "model": "llama-3.2-1b",
    "messages": [
      {"role": "user", "content": "Write a short poem about AI"}
    ],
    "stream": true
  }'
```

### Example 3: Multimodal Chat (with images)

```bash
# Encode image to base64
IMAGE_BASE64=$(base64 -w 0 image.jpg)

curl -X POST http://localhost:3000/api/chat \
  -H "Content-Type: application/json" \
  -d "{
    \"model\": \"llava-model\",
    \"messages\": [
      {
        \"role\": \"user\",
        \"content\": \"What is in this image?\",
        \"images\": [\"$IMAGE_BASE64\"]
      }
    ]
  }"
```

### Example 4: Using HTTPie (More Readable)

```bash
# Install httpie: pip install httpie

# Chat
http POST localhost:3000/api/chat \
  model=llama-3.2-1b \
  messages:='[{"role": "user", "content": "Hello!"}]' \
  temperature=0.7

# Streaming
http --stream POST localhost:3000/api/chat \
  model=llama-3.2-1b \
  messages:='[{"role": "user", "content": "Tell me a joke"}]' \
  stream:=true

# Embeddings
http POST localhost:3000/api/embed \
  model=embedding-model \
  input="Hello world"
```

### Example 5: Python Client

```python
import requests
import json

BASE_URL = "http://localhost:3000"

def chat(model, messages, stream=False, **kwargs):
    response = requests.post(
        f"{BASE_URL}/api/chat",
        json={"model": model, "messages": messages, "stream": stream, **kwargs},
        stream=stream
    )
    if stream:
        for line in response.iter_lines():
            if line:
                data = json.loads(line.decode().removeprefix("data: "))
                if data.get("message", {}).get("content"):
                    print(data["message"]["content"], end="", flush=True)
                if data.get("done"):
                    print()
                    break
    else:
        return response.json()

# Usage
messages = [
    {"role": "system", "content": "You are a helpful assistant."},
    {"role": "user", "content": "Explain quantum computing in one paragraph."}
]

# Non-streaming
result = chat("llama-3.2-1b", messages, max_tokens=200)
print(result["message"]["content"])

# Streaming
chat("llama-3.2-1b", messages, stream=True, max_tokens=200)
```

### Example 6: JavaScript/TypeScript Client

```javascript
const BASE_URL = 'http://localhost:3000';

async function chat(model, messages, options = {}) {
  const response = await fetch(`${BASE_URL}/api/chat`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ model, messages, ...options })
  });
  return response.json();
}

async function streamChat(model, messages, onToken) {
  const response = await fetch(`${BASE_URL}/api/chat`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ model, messages, stream: true })
  });

  const reader = response.body.getReader();
  const decoder = new TextDecoder();

  while (true) {
    const { done, value } = await reader.read();
    if (done) break;

    const text = decoder.decode(value);
    const lines = text.split('\n').filter(l => l.startsWith('data: '));

    for (const line of lines) {
      const data = JSON.parse(line.slice(6));
      if (data.message?.content) {
        onToken(data.message.content);
      }
      if (data.done) return;
    }
  }
}

// Usage
await chat('llama-3.2-1b', [
  { role: 'user', content: 'Hello!' }
]);

// Streaming
await streamChat('llama-3.2-1b', [
  { role: 'user', content: 'Write a haiku' }
], (token) => process.stdout.write(token));
```

### Example 7: Model Management

```bash
# List models
curl http://localhost:3000/api/tags | jq .

# Show model info
curl -X POST http://localhost:3000/api/show \
  -H "Content-Type: application/json" \
  -d '{"name": "llama-3.2-1b"}'

# List running models
curl http://localhost:3000/api/ps | jq .
```

### Example 8: Generate Text Completion

```bash
curl -X POST http://localhost:3000/api/generate \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llama-3.2-1b",
    "prompt": "The benefits of edge AI include",
    "max_tokens": 100,
    "temperature": 0.7
  }'
```

---

## Troubleshooting

### Common Issues

#### "Failed to open model: No such file or directory"
- Ensure model files exist in the configured `models_path`
- Check that model directory contains `.rkllm` file
- Verify config.yaml has correct `models_path`

#### "Failed to initialize RKLLM model: error code -1"
- Verify RKLLM runtime is installed (`librkllmrt.so`)
- Check NPU driver version compatibility
- Ensure model format matches RKLLM runtime version

#### "Connection refused" when accessing API
- Verify server is running (`rkllm-shell serve`)
- Check port 3000 is not blocked by firewall
- Try `curl http://localhost:3000/health`

#### Out of Memory / NPU Errors
- Try smaller model (1B or 0.5B parameters)
- Reduce `max_tokens` and context window
- Ensure no other NPU-intensive apps running

### Debug Logging

```bash
# Maximum verbosity
rkllm-shell -vvv serve

# Or set via environment
RUST_LOG=debug rkllm-shell serve

# Specific module logging
RUST_LOG=rkllm_shell::server=debug,rkllm_shell::commands=info rkllm-shell serve
```

### Log Levels

| Level | Description |
|-------|-------------|
| `error` | Only errors |
| `warn` | Warnings and errors |
| `info` | General info (default) |
| `debug` | Detailed debug info |
| `trace` | Very verbose tracing |

---

## Advanced Usage

### Custom Model Options

Pass advanced inference parameters via `options` object:

```json
{
  "model": "llama-3.2-1b",
  "messages": [...],
  "options": {
    "num_ctx": 4096,
    "repeat_last_n": 64,
    "repeat_penalty": 1.1,
    "temperature": 0.7,
    "seed": 42,
    "stop": ["\n", "<|endoftext|>"],
    "num_predict": 2048,
    "top_k": 40,
    "top_p": 0.9,
    "min_p": 0.0
  }
}
```

### Using with Docker

```dockerfile
FROM rust:1.75 as builder
WORKDIR /app
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/rkllm-shell /usr/local/bin/
COPY --from=builder /app/rkllm-api-sys/vendor/lib/aarch64/librkllmrt.so /usr/lib/
EXPOSE 3000
CMD ["rkllm-shell", "serve"]
```

```bash
# Build
docker build -t rkllm-shell .

# Run (requires NPU device access)
docker run --device=/dev/dri --device=/dev/rknpu -p 3000:3000 rkllm-shell
```

### Systemd Service

```ini
# /etc/systemd/system/rkllm-shell.service
[Unit]
Description=RKLLM Shell HTTP Server
After=network.target

[Service]
Type=simple
User=rkllm
WorkingDirectory=/opt/rkllm-shell
ExecStart=/usr/local/bin/rkllm-shell serve
Restart=on-failure
RestartSec=5
Environment=RUST_LOG=info

[Install]
WantedBy=multi-user.target
```

```bash
systemctl daemon-reload
systemctl enable --now rkllm-shell
```

### Reverse Proxy (Nginx)

```nginx
server {
    listen 80;
    server_name your-domain.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_read_timeout 300s;
        proxy_send_timeout 300s;
    }
}
```

### Performance Tuning

| Setting | Recommendation |
|---------|----------------|
| `num_ctx` | 2048-4096 for most models |
| `num_predict` | Limit to needed length |
| `temperature` | 0.7 for chat, 0.2 for factual |
| `top_p` | 0.9 for balanced sampling |
| Model quantization | Q4_K_M for best quality/size |

---

## API Compatibility

RKLLM Shell implements a subset of the **Ollama API** and **OpenAI API**:

| Endpoint | Ollama | OpenAI |
|----------|--------|--------|
| `/api/chat` | ✅ | - |
| `/api/generate` | ✅ | - |
| `/api/embed` | ✅ | - |
| `/api/tags` | ✅ | - |
| `/api/show` | ✅ | - |
| `/api/delete` | ✅ | - |
| `/api/pull` | ✅ | - |
| `/api/ps` | ✅ | - |
| `/v1/chat/completions` | - | ✅ |
| `/v1/models` | - | ✅ |

---

## Support

- **Issues**: [GitHub Issues](https://github.com/zerouid/rkllm-shell/issues)
- **Documentation**: `docs/` directory
- **API Examples**: `docs/tests.http`
- **Configuration**: See `config.yaml` in config directory

---

## Changelog

See [README.md](../README.md#changelog) for version history.