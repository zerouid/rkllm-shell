use super::pull::*;
use crate::config::Config;

#[test]
fn test_run_with_invalid_model() {
    std::env::set_var("HF_MODEL_ID", "nonexistent-model-xyz123");
    let config = Config::default();
    let args = Args::default();
    let result = run(&config, &args);
    assert!(result.is_err());
}
