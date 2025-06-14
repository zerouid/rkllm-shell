#![allow(non_upper_case_globals, dead_code)]

use std::collections::HashMap;
use std::ffi::{CStr, CString};
use std::sync::mpsc::{Sender, Receiver};
use std::sync::{mpsc, Arc, Mutex};

use rkllm_api_sys::{rkllm_createDefaultParam, rkllm_init, rkllm_run, rkllm_set_chat_template, LLMCallState, LLMCallState_RKLLM_RUN_ERROR, LLMCallState_RKLLM_RUN_FINISH, LLMCallState_RKLLM_RUN_NORMAL, LLMCallState_RKLLM_RUN_WAITING, LLMHandle, RKLLMInferMode_RKLLM_INFER_GENERATE, RKLLMInferParam, RKLLMInput, RKLLMInputType_RKLLM_INPUT_PROMPT, RKLLMInput__bindgen_ty_1, RKLLMResult};

use  crate::server::api_models::{ChatCompletionRequest, GenerateRequest};

pub enum CompletionRequest {
    Generate(GenerateRequest),
    Chat(ChatCompletionRequest),
}

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


#[derive(Debug)]
pub struct RkllmModel {
    // Wrap the unsafe pointer in a thread-safe container
    handle: ThreadSafeLLMHandle,
}

impl RkllmModel {
    pub fn run_inference(&self, messages: Vec<String>) -> Result<Receiver<String>, String> {
        let combined_msg = messages.join("\n");
        // let truncated_msg = if combined_msg.len() > 1000 { // Rough character limit
        //     format!("{}...", &combined_msg[..1000])
        // } else {
        //     combined_msg
        // };
        
        let msgs_cstr = CString::new(combined_msg).expect("CString::new failed");
        let mut rkllm_input = RKLLMInput {
            input_type: RKLLMInputType_RKLLM_INPUT_PROMPT,
            __bindgen_anon_1: RKLLMInput__bindgen_ty_1 { 
                prompt_input: msgs_cstr.as_ptr()
            },  
        };
        let mut rkllm_infer_params = RKLLMInferParam {
            mode: RKLLMInferMode_RKLLM_INFER_GENERATE,
            keep_history: 0,
            prompt_cache_params: std::ptr::null_mut(),
            lora_params: std::ptr::null_mut(),
        };
        
        let (tx, rx): (Sender<String>, Receiver<String>) = mpsc::channel();
        let sender_ptr = Box::into_raw(Box::new(tx)) as *mut ::std::os::raw::c_void;
        
        unsafe { 
            let result = rkllm_run(
                self.handle.as_llm_handle(), 
                &mut rkllm_input, 
                &mut rkllm_infer_params, 
                sender_ptr
            );
            
            if result != 0 {
                // Clean up the sender if call failed
                let _sender = Box::from_raw(sender_ptr as *mut Sender<String>);
                return Err(format!("rkllm_run failed with code: {}", result));
            }
        }
        
        // Keep CString alive until after the call
        std::mem::forget(msgs_cstr);
        
        Ok(rx)
    }
}

#[derive(Debug, Clone)]
pub struct RkllmRuntime {
    running_models: Arc<Mutex<HashMap<String, Arc<RkllmModel>>>>,
}


impl RkllmRuntime {
    pub fn new() -> Self {
        RkllmRuntime {
            running_models: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn get_request_model(&self, request: &CompletionRequest) -> Result<Arc<RkllmModel>, String> {
        let key = Self::parse_request_model_key(&request);
        
        let mut models = self.running_models.lock().unwrap();
        
        let m = models.entry(key).or_insert_with(|| {
            let h = self.init_model(&request).expect("Failed to initialize model" );
            Arc::new(RkllmModel {
                handle: ThreadSafeLLMHandle::new(h),
            })
        }).clone();
        Ok(m)
    }

    fn init_model(&self, request: &CompletionRequest) -> Result<LLMHandle, String> {
        let (model, options) = match request {
            CompletionRequest::Generate(req) => (&req.model, &req.options),
            CompletionRequest::Chat(req) => (&req.model, &req.options)
        };
        let mut param = unsafe { rkllm_createDefaultParam() };
        let model_path = self.get_model_path(model);
        let model_path_cstr = CString::new(model_path).expect("CString::new failed");
        param.model_path = model_path_cstr.as_ptr();
        param.top_k = options.top_k;
        param.top_p = options.top_p;
        param.temperature = options.temperature;
        param.repeat_penalty = options.repeat_penalty;
        param.frequency_penalty = 0.0;
        param.presence_penalty = 0.0;

        param.max_new_tokens = options.num_predict;
        // Fix: Use model's actual context limit instead of user request
        param.max_context_len = options.num_ctx; // Match model's actual limit
        param.skip_special_token = true;
        param.extend_param.base_domain_id = 0;
        param.extend_param.embed_flash = 1;
        
        let mut handle: LLMHandle = std::ptr::null_mut();
        
        // Fix: Create callback function pointer
        let callback_fn: unsafe extern "C" fn(*mut RKLLMResult, *mut ::std::os::raw::c_void, LLMCallState) = Self::llm_result_callback;
        
        unsafe { 
            let result = rkllm_init(&mut handle, &mut param, Some(callback_fn));
            if result != 0 {
                return Err(format!("Failed to initialize RKLLM model: error code {}", result));
            }
            
            // Fix: Use consistent ASCII characters for templates
            let system_template = CString::new("<|System|>").unwrap();
            let user_template = CString::new("<|User|>").unwrap();
            let assistant_template = CString::new("<|Assistant|>").unwrap();
            
            rkllm_set_chat_template(
                handle, 
                system_template.as_ptr(),
                user_template.as_ptr(), 
                assistant_template.as_ptr()
            );
        }
        
        // Keep CStrings alive until after C calls
        std::mem::forget(model_path_cstr);
        
        Ok(handle)
    }

    fn get_model_path(&self, model: &str) -> String {
        // This function should return the path to the model based on the model name
        // For now, we will just return a placeholder path
        // format!("/path/to/models/{}", model)
        "/home/vanko/models/DeepSeek-R1-Distill-Qwen-1.5B_W8A8_RK3588.rkllm".to_string()
    }

    fn parse_request_model_key(request: &CompletionRequest) -> String {
        let (model, options) = match request {
            CompletionRequest::Generate(req) => (&req.model, &req.options),
            CompletionRequest::Chat(req) => (&req.model, &req.options)
        };
        format!("{}-{}-{}-{}-{}-{}-{}-{}-{}-{}-{}", 
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
                options.min_p)
    }

    pub extern "C" fn llm_result_callback(
        result: *mut RKLLMResult, 
        userdata: *mut ::std::os::raw::c_void, 
        state: LLMCallState
    ) {
        // Safety check: ensure pointers are valid
        if userdata.is_null() {
            eprintln!("Error: userdata is null in callback");
            return;
        }

        // Don't take ownership yet - we might need to keep the sender alive
        let sender_ptr = userdata as *mut Sender<String>;
        
        let (response, should_end) = match state {
            LLMCallState_RKLLM_RUN_FINISH => {
                (String::from(""), true) // Empty response on finish
            },
            LLMCallState_RKLLM_RUN_ERROR => {
                (String::from("[ERROR]"), true)
            },
            LLMCallState_RKLLM_RUN_WAITING => {
                (String::from(""), false) // No response while waiting
            },
            LLMCallState_RKLLM_RUN_NORMAL => {
                if result.is_null() {
                    (String::from("[NULL_RESULT]"), false)
                } else {
                    unsafe {
                        let text_ptr = (*result).text;
                        if text_ptr.is_null() {
                            (String::from("[NULL_TEXT]"), false)
                        } else {
                            let cstr = CStr::from_ptr(text_ptr);
                            match cstr.to_str() {
                                Ok(s) => (s.to_string(), false),
                                Err(_) => (String::from("[INVALID_UTF8]"), false),
                            }
                        }
                    }
                }
            },
            _ => {
                (format!("[UNKNOWN_STATE_{}]", state), true)
            }
        };

        // Only send non-empty responses
        if !response.is_empty() {
            unsafe {
                let sender = &*sender_ptr;
                if let Err(e) = sender.send(response) {
                    eprintln!("Failed to send response: {}", e);
                }
            }
        }

        // Clean up the sender only when we're done
        if should_end {
            unsafe {
                let _sender = Box::from_raw(sender_ptr);
                // Sender will be dropped here, closing the channel
            }
        }
    }

    // pub async fn generate_completion(&self, body: GenerateRequest) -> GenerateResponse {
    //     // Implement the logic to generate a completion
    //     unimplemented!()
    // }
}