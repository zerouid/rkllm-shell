use std::time::Duration;

pub (crate) fn default_context_window() -> i32 {
    2048
}

pub (crate) fn default_repeat_last_n() -> i32 {
    64
}

pub (crate) fn default_repeat_penalty() -> f32 {
    1.1
}

pub (crate) fn default_temperature() -> f32 {
    0.8
}

pub (crate) fn default_seed() -> i32 {
    0
}

pub (crate) fn default_num_predict() -> i32 {
    -1
}

pub (crate) fn default_top_k() -> i32 {
    40
}

pub (crate) fn default_top_p() -> f32 {
    0.9
}

pub (crate) fn default_stop() -> Vec<String> {
    vec![]
}

pub (crate) fn default_min_p() -> f32 {
    0.0
}

pub (crate) fn default_stream() -> bool {
    false
}

pub (crate) fn default_keep_alive() -> Duration {
    Duration::from_secs(300) // 5 minutes    
}

pub (crate) fn default_raw() -> bool {
    false
}

pub (crate) fn default_think() -> bool {
    false
}

pub (crate) fn default_embed_truncation() -> bool {
    false
}