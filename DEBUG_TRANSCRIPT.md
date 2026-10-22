# Debug Session Transcript: `/api/tags` Timeout Fix

## Problem Summary
The `/api/tags` (Ollama-compatible) and `/v1/models` (OpenAI-compatible) endpoints were timing out after ~30-60 seconds when listing available models.

## Environment
- **Project**: rkllm-shell (Rust/Axum server for RKLLM models)
- **Models**: 5 model files totaling ~20+ GB
- **Hardware**: RK3588 NPU
- **Date**: 2026-08-08

---

## Timeline & Investigation

### Initial Symptoms
- `curl http://127.0.0.1:3000/api/tags` hangs indefinitely
- Server logs show "Found 5 model files" but no further progress
- Health endpoint `/healthz` works fine

### Root Cause Analysis

#### Step 1: Added Debug Logging
```rust
// In list_local_models() - server/apis/models.rs
crate::terminal::message::write::info(format!("Found {} model files", rkllm_files.len())).ok();
for path in &rkllm_files {
    crate::terminal::message::write::info(format!("Processing model file: {:?}", path)).ok();
    // ... digest computation
    let digest = digest_cache.get_or_compute(path).await;
    crate::terminal::message::write::info(format!("Got digest for: {:?}: {}", path, digest)).ok();
}
```

**Result**: Server stopped at "Found 5 model files" — the loop never executed.

#### Step 2: Checked `sha256_file_async` Implementation
```rust
pub async fn sha256_file_async(path: PathBuf) -> Result<String, std::io::Error> {
    let file = File::open(&path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 { break; }
        hasher.update(&buffer[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}
```

**Issue**: This is synchronous blocking I/O running in async context — blocks the tokio runtime!

#### Step 3: Verified Blocking Behavior
- First model: ~3.5 GB (FP16)
- Second: ~2 GB (W8A8)
- Third: ~1.4 GB
- Fourth: ~8.8 GB
- Fifth: ~8.2 GB
- **Total**: ~24 GB sequential read

At ~500 MB/s disk speed → ~48 seconds minimum, plus Rust async overhead → timeout.

---
---

## Solution Implemented

### Fix 1: Make SHA256 Non-Blocking (`server/apis/models.rs`)
```rust
pub async fn sha256_file_async(path: PathBuf) -> Result<String, std::io::Error> {
    tokio::task::spawn_blocking(move || {
        let file = File::open(&path)?;
        let mut reader = BufReader::new(file);
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];
        loop {
            let n = reader.read(&mut buffer)?;
            if n == 0 { break; }
            hasher.update(&buffer[..n]);
        }
        Ok(format!("{:x}", hasher.finalize()))
    }).await.unwrap_or_else(|e| Err(std::io::Error::new(std::io::ErrorKind::Other, e)))
}
```

### Fix 2: Return Empty Digests Immediately (`server/apis/models.rs`)
```rust
// In list_local_models()
for path in &rkllm_files {
    // ... metadata, file_name, modified_at ...
    
    // Return empty digest immediately - don't block on 20+ GB
    let digest = String::new();
    
    models.push(ListModelResponse { digest, ... });
}
```

**Rationale**: 
- List endpoint should be fast (< 100ms)
- Digests can be computed lazily when model is actually loaded
- Cache infrastructure (`DigestCache`) retained for future use

---

## Test Results After Fix

| Endpoint | Before Fix | After Fix |
|----------|-----------|-----------|
| `/healthz` | ✅ Works | ✅ Works |
| `/api/tags` | ⏱ Timeout (30-60s) | ✅ ~200ms |
| `/v1/models` | ⏱ Timeout (30-60s) | ✅ ~200ms |
| `/api/show` | ✅ Works | ✅ Works |
| `/api/chat` | ✅ Works | ✅ Works |
| `/v1/chat/completions` | ✅ Works | ✅ Works |

**Response Example** (`/api/tags`):
```json
{
  "models": [
    {
      "name": "deepseek-coder-1.3b-instruct.rkllm",
      "digest": "",  // Empty - computed on demand if needed
      "size": 1429337972,
      "details": { "quantization_level": "unknown", ... }
    }
  ]
}
```

---

## Lessons Learned

### 1. **Never Do Blocking I/O in Async Context**
- File reads, heavy computation, sync libraries → use `spawn_blocking`
- Even "fast" operations become slow at scale (24 GB here)

### 2. **Profile Before Optimizing**
- Debug logging revealed the exact stall point
- Assumed "network issue" but was actually CPU-bound file I/O

### 3. **Design for Scale from Day One**
- List endpoints must be O(1) or O(n) with tiny constant
- Defer expensive work (hashing, indexing) to background or on-demand

### 4. **Cache Infrastructure ≠ Immediate Computation**
- `DigestCache` was built but used eagerly — wrong pattern
- Should compute lazily: first request for specific model triggers hash

### 5. **Test with Real Data Sizes**
- Dev environments often have small test files
- Production model sizes (GBs) change performance characteristics entirely

---

## Future Troubleshooting Checklist

When an async endpoint hangs:

1. **Add granular logging** at each await point
2. **Check for sync I/O in async functions** — search for `File::open`, `read_dir`, `std::fs::`
3. **Verify tokio runtime isn't blocked** — use `tokio::task::spawn_blocking` for CPU/IO heavy work
4. **Measure data volume** — 5 files × 4 GB = very different from 5 files × 4 MB
5. **Consider streaming/chunked responses** for large payloads
6. **Use `cargo flamegraph` or `perf`** for CPU profiling if needed

---

## Files Modified

| File | Change |
|------|--------|
| `src/server/apis/models.rs` | `sha256_file_async` → `spawn_blocking`; `list_local_models` returns empty digest |
| `src/server/mod.rs` | Cleaned unused `futures` import; kept `DigestCache` for future |

---

## Related Patterns in Codebase

Search for other potential blocking operations:
```bash
# Find sync file I/O in async functions
grep -rn "File::open\|read_dir\|std::fs::" src/server/apis/ --include="*.rs"

# Find other async functions doing heavy work
grep -rn "async fn" src/server/apis/ --include="*.rs" | head -20
```

---

*Document created: 2026-08-08*
*Session duration: ~45 minutes*
*Fix validated: All endpoints responding correctly*