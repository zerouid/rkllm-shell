# RKLLM-Shell Roadmap

## Current Status (v0.1.0-devel)
- ✅ Core RKLLM runtime integration
- ✅ OpenAI-compatible API endpoints (/chat, /generate, /embed, /models)
- ✅ Ollama-compatible API endpoints
- ✅ Agent API with chat and streaming endpoints
- ✅ Comprehensive test suite (31 tests passing)
- ✅ Mock runtime for testing
- ✅ Vision model support

---

## 🚀 Immediate (Next Sprint)

### Testing & Quality
- [ ] **Add mock model files** for integration testing with actual model loading
  - Create minimal .rkllm model files in `./mock_models/`
  - Update integration tests to verify successful model inference
  - Target: `test_agent_chat_endpoint_structure` returns 200 with valid response

- [ ] **Add CLI command tests** for `agent.rs`
  - Test `rkllm-shell agent chat` command
  - Test `rkllm-shell agent stream` command
  - Verify help text and argument parsing

- [ ] **Increase test coverage** to >80%
  - Current: ~60% (estimated)
  - Focus on: `rig_provider.rs`, `rkllm_runtime.rs`, API handlers

### Documentation
- [ ] **Add OpenAPI/Swagger documentation** for all endpoints
  - Annotate handlers with `utoipa`/`aide` derives
  - Generate `openapi.json` at build time
  - Serve Swagger UI at `/docs` or `/swagger-ui`

- [ ] **Add README examples** for Agent API
  - cURL examples for `/api/agent/chat`
  - cURL examples for `/api/agent/stream`
  - Python/JS client examples

### Code Quality
- [ ] **Address clippy warnings** (105 warnings currently)
  - Run `cargo fix --bin rkllm-shell` for auto-fixable
  - Manually address remaining warnings
  - Add `clippy::pedantic` to CI

- [ ] **Add pre-commit hooks**
  - `cargo fmt --check`
  - `cargo clippy -- -D warnings`
  - `cargo test`

---

## 📋 Short Term (1-2 Months)

### API Enhancements
- [ ] **Request validation & rate limiting**
  - Add input validation (max tokens, message count, etc.)
  - Implement rate limiting per IP/client
  - Add request size limits

- [ ] **Authentication & Authorization**
  - API key authentication
  - Optional: JWT/OAuth2 support
  - Role-based access (admin, user)

- [ ] **Structured output / JSON mode**
  - Add `response_format` parameter (json_schema, json_object)
  - Validate output against schema
  - Support for function calling / tool use

- [ ] **Conversation management**
  - Session persistence (Redis/SQLite)
  - Conversation history API
  - Context window management

### Performance
- [ ] **Model caching & pooling**
  - Keep multiple models loaded simultaneously
  - LRU eviction for memory management
  - Pre-warm models on startup

- [ ] **Streaming optimizations**
  - Reduce first-token latency
  - Implement proper backpressure
  - Add cancellation support

- [ ] **Batch inference**
  - Support multiple prompts in single request
  - Optimize for throughput scenarios

### Operations
- [ ] **Health checks & metrics**
  - `/health` endpoint with model status
  - Prometheus metrics export
  - Structured logging (JSON)

- [ ] **Docker & deployment**
  - Multi-arch Docker image (amd64, arm64)
  - Docker Compose for development
  - Kubernetes deployment manifests
  - RKNPU device plugin configuration

---

## 🎯 Medium Term (3-6 Months)

### Advanced Features
- [ ] **Function Calling / Tool Use**
  - OpenAI-compatible `tools` parameter
  - Built-in tools (web search, code execution, file ops)
  - Custom tool registration API

- [ ] **RAG (Retrieval-Augmented Generation)**
  - Document ingestion API
  - Vector store integration (Qdrant, Chroma, SQLite-vec)
  - Hybrid search (keyword + semantic)
  - Citation/grounding in responses

- [ ] **Multi-modal enhancements**
  - Video input support
  - Audio input/output (speech-to-text, text-to-speech)
  - Image generation integration

- [ ] **Model management UI**
  - Web dashboard for model loading/unloading
  - Model download progress
  - Resource usage monitoring

### Ecosystem
- [ ] **Plugin system**
  - Dynamic model loaders
  - Custom preprocessing/postprocessing
  - Middleware support

- [ ] **Client SDKs**
  - Python SDK (PyPI)
  - JavaScript/TypeScript SDK (npm)
  - Go SDK
  - Rust SDK (this crate)

- [ ] **Compatibility layers**
  - vLLM API compatibility
  - TGI (Text Generation Inference) compatibility
  - LiteLLM proxy support

---

## 🔮 Long Term (6+ Months)

### Architecture
- [ ] **Distributed inference**
  - Model sharding across multiple RKNPU devices
  - Multi-node cluster support
  - Load balancing

- [ ] **Quantization & optimization**
  - Dynamic quantization (INT4, INT8)
  - Speculative decoding
  - KV cache optimization

- [ ] **Fine-tuning API**
  - LoRA/QLoRA training endpoint
  - Dataset management
  - Model export to RKLLM format

### Platform
- [ ] **Model marketplace/registry**
  - Curated RKLLM model repository
  - Version management
  - Auto-update capability

- [ ] **Enterprise features**
  - Audit logging
  - Data residency controls
  - SLA monitoring
  - Multi-tenancy

---

## 📝 Maintenance Tasks (Ongoing)

### Weekly
- [ ] Review and triage GitHub issues
- [ ] Update dependencies (`cargo update`)
- [ ] Run security audit (`cargo audit`)

### Monthly
- [ ] Benchmark performance regressions
- [ ] Review test coverage report
- [ ] Update documentation for new features

### Quarterly
- [ ] Major dependency updates
- [ ] Architecture review
- [ ] Security review
- [ ] Release planning

---

## 🏷️ Version Milestones

| Version | Target Date | Key Features |
|---------|-------------|--------------|
| v0.2.0 | Q1 2025 | OpenAPI docs, auth, rate limiting, Docker |
| v0.3.0 | Q2 2025 | Function calling, RAG, streaming optimizations |
| v0.4.0 | Q3 2025 | Multi-modal, model management UI, plugins |
| v1.0.0 | Q4 2025 | Distributed inference, fine-tuning, enterprise |

---

## 🤝 Contribution Guidelines

See [CONTRIBUTING.md](CONTRIBUTING.md) for:
- Code style guide
- Testing requirements
- PR review process
- Release process

---

## 📊 Metrics & KPIs

| Metric | Current | Target |
|--------|---------|--------|
| Test Coverage | ~60% | >80% |
| Clippy Warnings | 105 | 0 |
| First Token Latency | ~500ms | <200ms |
| Throughput (tok/s) | TBD | >50 |
| API Compatibility | OpenAI/Ollama | +vLLM/TGI |
| Supported Platforms | Linux RK3588 | +macOS, Windows, AMD64 |

---

## 🔗 Related Resources

- [RKLLM Runtime Documentation](https://github.com/rockchip-linux/rkllm)
- [Rig Library](https://github.com/0xPlaygrounds/rig)
- [Axum Web Framework](https://github.com/tokio-rs/axum)
- [OpenAPI Specification](https://spec.openapis.org/oas/latest.html)
