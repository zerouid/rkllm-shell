#![allow(non_upper_case_globals, dead_code)]

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use rkllm_api_sys::{
    rkllm_createDefaultParam, rkllm_destroy, rkllm_init, rkllm_run, rkllm_set_chat_template,
    LLMCallState, LLMCallState_RKLLM_RUN_ERROR, LLMCallState_RKLLM_RUN_FINISH,
    LLMCallState_RKLLM_RUN_NORMAL, LLMCallState_RKLLM_RUN_WAITING, LLMHandle,
    RKLLMInferMode_RKLLM_INFER_GENERATE, RKLLMInferParam, RKLLMInput,
    RKLLMInputType_RKLLM_INPUT_PROMPT, RKLLMInput__bindgen_ty_1, RKLLMResult,
};

use crate::server::api_models::{ChatCompletionRequest, GenerateRequest};

pub enum CompletionRequest {
    Generate(GenerateRequest),
    Chat(ChatCompletionRequest),
}

impl CompletionRequest {
    pub fn keep_alive(&self) -> Duration {
        match self {
            CompletionRequest::Generate(r) => r.keep_alive,
            CompletionRequest::Chat(r) => r.keep_alive,
        }
    }
}

// ---------------------------------------------------------------------------
// Thread-safe wrapper around the raw LLMHandle pointer
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct ThreadSafeLLMHandle(LLMHandle);

unsafe impl Send for ThreadSafeLLMHandle {}
unsafe impl Sync for ThreadSafeLLMHandle {}

impl ThreadSafeLLMHandle {
    pub fn new(handle: LLMHandle) -> Self {
        ThreadSafeLLMHandle(handle)
    }

    pub fn as_llm_handle(&self) -> LLMHandle {
        self.0
    }
}

// Used to move an LLMHandle across thread boundaries inside spawn_blocking closures.
struct RawHandleSend(LLMHandle);
unsafe impl Send for RawHandleSend {}

// ---------------------------------------------------------------------------
// RkllmModel — owns the native handle; destroys it on drop.
// ---------------------------------------------------------------------------

#[derive(Debug)]
pub struct RkllmModel {
    handle: ThreadSafeLLMHandle,
}

impl Drop for RkllmModel {
    fn drop(&mut self) {
        let h = self.handle.as_llm_handle();
        if !h.is_null() {
            unsafe {
                rkllm_destroy(h);
            }
        }
    }
}

impl RkllmModel {
    /// Runs inference in a blocking thread so the tokio executor is never stalled.
    /// Returns an async-compatible receiver that yields token strings.
    pub fn run_inference(
        &self,
        messages: Vec<String>,
    ) -> tokio::sync::mpsc::UnboundedReceiver<String> {
        let combined_msg = messages.join("\n");
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<String>();

        // Convert the handle and sender box-pointer to `usize` so the closure
        // is `Send + 'static` (raw pointers are neither).
        let handle_usize = self.handle.as_llm_handle() as usize;
        // Box the sender and capture its address as usize.
        let tx_ptr_usize = Box::into_raw(Box::new(tx)) as usize;

        tokio::task::spawn_blocking(move || {
            // Restore typed pointers inside the blocking thread.
            let handle = handle_usize as LLMHandle;
            let sender_ptr = tx_ptr_usize as *mut ::std::os::raw::c_void;

            let msgs_cstr = CString::new(combined_msg).expect("CString::new failed");
            let mut rkllm_input = RKLLMInput {
                input_type: RKLLMInputType_RKLLM_INPUT_PROMPT,
                __bindgen_anon_1: RKLLMInput__bindgen_ty_1 {
                    prompt_input: msgs_cstr.as_ptr(),
                },
            };
            let mut rkllm_infer_params = RKLLMInferParam {
                mode: RKLLMInferMode_RKLLM_INFER_GENERATE,
                keep_history: 0,
                prompt_cache_params: std::ptr::null_mut(),
                lora_params: std::ptr::null_mut(),
            };

            unsafe {
                let result = rkllm_run(
                    handle,
                    &mut rkllm_input,
                    &mut rkllm_infer_params,
                    sender_ptr,
                );

                if result != 0 {
                    // Clean up sender — dropping it closes the channel so the
                    // receiver sees EOF.
                    let _sender = Box::from_raw(
                        sender_ptr
                            as *mut tokio::sync::mpsc::UnboundedSender<String>,
                    );
                }
            }
            // msgs_cstr kept alive until here (past the rkllm_run call).
            drop(msgs_cstr);
        });

        rx
    }
}

// ---------------------------------------------------------------------------
// Per-entry metadata kept alongside the Arc<RkllmModel>
// ---------------------------------------------------------------------------

struct ModelEntry {
    // Note: AbortHandle implements Debug; RkllmModel implements Debug.
    model: Arc<RkllmModel>,
    eviction_handle: tokio::task::AbortHandle,
}

// ---------------------------------------------------------------------------
// RkllmRuntime — manages loaded models and their lifecycle.
// ---------------------------------------------------------------------------

#[derive(Clone)]
pub struct RkllmRuntime {
    running_models: Arc<Mutex<HashMap<String, ModelEntry>>>,
    models_path: Arc<PathBuf>,
}

impl RkllmRuntime {
    pub fn new(models_path: PathBuf) -> Self {
        RkllmRuntime {
            running_models: Arc::new(Mutex::new(HashMap::new())),
            models_path: Arc::new(models_path),
        }
    }

    pub fn list_running_models(&self) -> Vec<String> {
        let models = self.running_models.lock().unwrap();
        // Return only the base model name (first component of the composite key)
        models
            .keys()
            .filter_map(|k| k.splitn(2, '-').next().map(str::to_string))
            .collect()
    }

    /// Returns (or lazily initialises) the model for the given request, then
    /// resets its keep-alive eviction timer.
    pub async fn get_request_model(
        &self,
        request: &CompletionRequest,
    ) -> Result<Arc<RkllmModel>, String> {
        let key = Self::parse_request_model_key(request);
        let keep_alive = request.keep_alive();

        // Fast-path: already loaded.
        {
            let mut models = self.running_models.lock().unwrap();
            if let Some(entry) = models.get_mut(&key) {
                // Reset eviction timer.
                entry.eviction_handle.abort();
                let eviction_handle =
                    self.spawn_eviction_task(key.clone(), keep_alive);
                entry.eviction_handle = eviction_handle;
                return Ok(entry.model.clone());
            }
        }

        // Cold-path: initialise in a blocking thread.
        let handle = self.init_model_async(request).await?;
        let model = Arc::new(RkllmModel {
            handle: ThreadSafeLLMHandle::new(handle),
        });

        let eviction_handle = self.spawn_eviction_task(key.clone(), keep_alive);

        {
            let mut models = self.running_models.lock().unwrap();
            // Another concurrent request might have beaten us — keep the winner.
            models.entry(key).or_insert(ModelEntry {
                model: model.clone(),
                eviction_handle,
            });
        }

        Ok(model)
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    fn spawn_eviction_task(
        &self,
        key: String,
        duration: Duration,
    ) -> tokio::task::AbortHandle {
        let map = self.running_models.clone();
        let join = tokio::spawn(async move {
            tokio::time::sleep(duration).await;
            let mut models = map.lock().unwrap();
            if let Some(entry) = models.remove(&key) {
                // Arc<RkllmModel> is dropped here → Drop calls rkllm_destroy
                // (unless another caller still holds a clone).
                drop(entry);
            }
        });
        join.abort_handle()
    }

    /// Runs `rkllm_init` in a blocking thread so the tokio executor is not
    /// stalled by the (potentially long) model-loading operation.
    async fn init_model_async(
        &self,
        request: &CompletionRequest,
    ) -> Result<LLMHandle, String> {
        let (model_name, options) = match request {
            CompletionRequest::Generate(r) => (&r.model, &r.options),
            CompletionRequest::Chat(r) => (&r.model, &r.options),
        };

        let model_path = self.get_model_path(model_name);
        let top_k = options.top_k;
        let top_p = options.top_p;
        let temperature = options.temperature;
        let repeat_penalty = options.repeat_penalty;
        let max_new_tokens = options.num_predict;
        let max_context_len = options.num_ctx;

        let result = tokio::task::spawn_blocking(move || {
            let model_path_cstr =
                CString::new(model_path).expect("CString::new failed");

            let mut param = unsafe { rkllm_createDefaultParam() };
            param.model_path = model_path_cstr.as_ptr();
            param.top_k = top_k;
            param.top_p = top_p;
            param.temperature = temperature;
            param.repeat_penalty = repeat_penalty;
            param.frequency_penalty = 0.0;
            param.presence_penalty = 0.0;
            param.max_new_tokens = max_new_tokens;
            param.max_context_len = max_context_len;
            param.skip_special_token = true;
            param.extend_param.base_domain_id = 0;
            param.extend_param.embed_flash = 1;

            let callback_fn: unsafe extern "C" fn(
                *mut RKLLMResult,
                *mut ::std::os::raw::c_void,
                LLMCallState,
            ) = RkllmRuntime::llm_result_callback;

            let mut handle: LLMHandle = std::ptr::null_mut();
            let init_result = unsafe {
                let r = rkllm_init(&mut handle, &mut param, Some(callback_fn));
                if r == 0 {
                    let sys = CString::new("<|System|>").unwrap();
                    let usr = CString::new("<|User|>").unwrap();
                    let ast = CString::new("<|Assistant|>").unwrap();
                    rkllm_set_chat_template(
                        handle,
                        sys.as_ptr(),
                        usr.as_ptr(),
                        ast.as_ptr(),
                    );
                }
                r
            };

            // Keep model_path_cstr alive past the C calls.
            drop(model_path_cstr);

            if init_result != 0 {
                return Err(format!(
                    "Failed to initialize RKLLM model: error code {}",
                    init_result
                ));
            }
            Ok(RawHandleSend(handle))
        })
        .await
        .map_err(|e| format!("spawn_blocking panicked: {}", e))??;

        Ok(result.0)
    }

    fn get_model_path(&self, model: &str) -> String {
        // If `model` already looks like an absolute path, use it directly.
        let p = Path::new(model);
        if p.is_absolute() {
            return model.to_string();
        }
        // Otherwise resolve relative to models_path.
        self.models_path.join(model).to_string_lossy().into_owned()
    }

    fn parse_request_model_key(request: &CompletionRequest) -> String {
        let (model, options) = match request {
            CompletionRequest::Generate(req) => (&req.model, &req.options),
            CompletionRequest::Chat(req) => (&req.model, &req.options),
        };
        format!(
            "{}-{}-{}-{}-{}-{}-{}-{}-{}-{}-{}",
            model,
            options.num_ctx,
            options.repeat_last_n,
            options.repeat_penalty,
            options.temperature,
            options.seed,
            options.num_predict,
            options.top_k,
            options.top_p,
            options.stop.join(","),
            options.min_p
        )
    }

    pub extern "C" fn llm_result_callback(
        result: *mut RKLLMResult,
        userdata: *mut ::std::os::raw::c_void,
        state: LLMCallState,
    ) {
        if userdata.is_null() {
            eprintln!("Error: userdata is null in callback");
            return;
        }

        let sender_ptr =
            userdata as *mut tokio::sync::mpsc::UnboundedSender<String>;

        let (response, should_end) = match state {
            LLMCallState_RKLLM_RUN_FINISH => (String::new(), true),
            LLMCallState_RKLLM_RUN_ERROR => ("[ERROR]".to_string(), true),
            LLMCallState_RKLLM_RUN_WAITING => (String::new(), false),
            LLMCallState_RKLLM_RUN_NORMAL => {
                if result.is_null() {
                    ("[NULL_RESULT]".to_string(), false)
                } else {
                    unsafe {
                        let text_ptr = (*result).text;
                        if text_ptr.is_null() {
                            ("[NULL_TEXT]".to_string(), false)
                        } else {
                            let cstr = CStr::from_ptr(text_ptr);
                            match cstr.to_str() {
                                Ok(s) => (s.to_string(), false),
                                Err(_) => ("[INVALID_UTF8]".to_string(), false),
                            }
                        }
                    }
                }
            }
            _ => (format!("[UNKNOWN_STATE_{}]", state), true),
        };

        if !response.is_empty() {
            unsafe {
                let sender = &*sender_ptr;
                if let Err(e) = sender.send(response) {
                    eprintln!("Failed to send token: {}", e);
                }
            }
        }

        if should_end {
            unsafe {
                // Drop the sender → closes the channel → receiver sees EOF.
                let _sender = Box::from_raw(
                    sender_ptr
                        as *mut tokio::sync::mpsc::UnboundedSender<String>,
                );
            }
        }
    }
}

// Ensure ModelEntry is not inadvertently made Sync (the raw handle is not).
// The HashMap is behind Arc<Mutex<…>> so this is fine.
unsafe impl Send for ModelEntry {}

#[cfg(test)]
#[path = "rkllm_runtime_test.rs"]
mod tests;