# rkllm-shell — Prioritized TODO

## Phase 1 — Correctness (P0) ✅

- [x] **1.1 — Unified `AppState`:** Created `AppState { runtime: RkllmRuntime, config: Arc<Config> }` replacing `RkllmRuntime` as the axum router state. All handlers updated to use `State<AppState>`.
- [x] **1.2 — Fix hardcoded model paths:** `RkllmRuntime::get_model_path` now resolves against `config.models_path`. All `apis/models.rs` handlers use `state.config.models_path`.
- [x] **1.3 — NPU cleanup (`rkllm_destroy`):** `Drop` implemented for `RkllmModel` — calls `rkllm_destroy` on the handle. Model eviction (keep_alive) also triggers cleanup via `Arc` drop.
- [x] **1.4 — Async executor safety:** `rkllm_init` wrapped in `spawn_blocking` in `init_model_async`. `rkllm_run` wrapped in `spawn_blocking` in `run_inference`. Handle passed as `usize` to satisfy `Send + 'static`.

## Phase 2 — Core Feature Completion (P1) ✅

- [x] **2.1 — Implement `/api/generate`:** `apis/generate.rs` fully implemented — mirrors chat handler, supports both streaming and non-streaming.
- [x] **2.2 — SSE Streaming:** `/api/chat` and `/api/generate` both support `stream: true` via `axum::response::sse::Sse`. Each token is emitted as a JSON SSE event; a final `done: true` event closes the stream.
- [x] **2.3 — `/v1/chat/completions`:** OpenAI-compatible endpoint added. `OpenAiChatRequest` / `OpenAiChatResponse` / chunk types defined in `api_models.rs`. Supports both streaming (`data: [DONE]` sentinel) and non-streaming responses.
- [x] **2.4 — `keep_alive` model eviction:** Per-request `keep_alive` duration schedules a `tokio::spawn` eviction task. Timer is reset on subsequent requests for the same model key. Eviction removes from `running_models`; `Drop` calls `rkllm_destroy`.

## Phase 3 — Model Management (P2) ✅

- [x] **3.1 — Fix `pull_model` download target:** Download destination resolved from `config.models_path`. HF cache → models_path copy performed in `spawn_blocking`.
- [x] **3.2 — Quantization detection:** `detect_quantization()` parses `W4A16`, `W8A8`, `W4A8`, `W8A16`, `INT4`, `INT8`, `FP16`, `FP32` from model filenames.
- [x] **3.3 — SHA256 digest:** `sha256_file()` computes streaming SHA-256 of each `.rkllm` file; `ListModelResponse.digest` populated as `sha256:<hex>`.
- [x] **3.4 — Wire CLI commands:** `list`, `pull`, `show`, and `ps` commands now call the local HTTP API via `reqwest` and display structured output.

## Phase 4 — Polish & Extended Features (P3)

- [ ] **4.1 — OpenAI schema separation:** Keep Ollama and OpenAI request/response types in separate modules (`api_models/ollama.rs`, `api_models/openai.rs`) with a translation layer (`api_models/translate.rs`) converting between the two.
- [ ] **4.2 — HF model search endpoint:** Add `GET /api/search?q=<query>` that queries the Hugging Face Hub for models filtered by the `rkllm` tag using the `hf-hub` crate.
- [ ] **4.3 — Extended `ApiError`:** Add `HfError(String)` and `FfiError(String)` variants to `ApiError` and map HF hub errors and FFI failures through them.
- [ ] **4.4 — Unit & integration tests:** Add/expand tests for `RkllmRuntime` (mock FFI), model path resolution, quantization parsing, and OpenAI↔Ollama schema translation.

---

## Reference Gaps Status

| File | Issue | Status |
|---|---|---|
| `rkllm_runtime.rs` | `get_model_path()` hardcoded path | ✅ Fixed |
| `apis/models.rs` | `PathBuf::from("./data")` instead of `config.models_path` | ✅ Fixed |
| `server/mod.rs` | Only `RkllmRuntime` passed as state | ✅ Fixed — `AppState` |
| `rkllm_runtime.rs` | `rkllm_destroy` never called | ✅ Fixed — `Drop` impl |
| `rkllm_runtime.rs` | `rkllm_init` / `rkllm_run` block the tokio executor | ✅ Fixed — `spawn_blocking` |
| `apis/chat.rs` | Always buffers full response; `stream: true` ignored | ✅ Fixed — SSE |
| `apis/generate.rs` | `unimplemented!()` | ✅ Fixed |
| `apis/embed.rs` | `unimplemented!()` | ✅ Stub returns empty embeddings |
| `server/mod.rs` | No `/v1/chat/completions` route | ✅ Fixed |
| `apis/models.rs` | `quantization_level` hardcoded `"int8"` | ✅ Fixed — detection |
| `apis/models.rs` | `digest` is always empty string | ✅ Fixed — SHA-256 |
| `commands/pull.rs` etc. | CLI commands are no-op stubs | ✅ Fixed |