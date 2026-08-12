use std::time::Duration;

use crate::server::ollama_models::ModelOptions;

pub fn default_context_window() -> i32 {
    2048
}

pub fn default_repeat_last_n() -> i32 {
    64
}

pub fn default_repeat_penalty() -> f32 {
    1.1
}

pub fn default_temperature() -> f32 {
    0.8
}

pub fn default_seed() -> i32 {
    0
}

pub fn default_num_predict() -> i32 {
    -1
}

pub fn default_top_k() -> i32 {
    40
}

pub fn default_top_p() -> f32 {
    0.9
}

pub fn default_stop() -> Vec<String> {
    vec![]
}

pub fn default_min_p() -> f32 {
    0.0
}

pub fn default_stream() -> bool {
    false
}

pub fn default_insecure() -> bool {
    false
}

pub fn default_keep_alive() -> Duration {
    Duration::from_secs(300)
}

pub fn default_raw() -> bool {
    false
}

pub fn default_think() -> bool {
    false
}

pub fn default_embed_truncation() -> bool {
    false
}

pub fn default_model_options() -> ModelOptions {
    ModelOptions {
        num_ctx: default_context_window(),
        repeat_last_n: default_repeat_last_n(),
        repeat_penalty: default_repeat_penalty(),
        temperature: default_temperature(),
        seed: default_seed(),
        stop: default_stop(),
        num_predict: default_num_predict(),
        top_k: default_top_k(),
        top_p: default_top_p(),
        min_p: default_min_p(),
    }
}
