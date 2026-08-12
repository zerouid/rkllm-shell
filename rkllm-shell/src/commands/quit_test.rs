#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[tokio::test]
    async fn test_quit_command() {
        // Create a minimal config for testing
        let config = Config::default();
        let args = Args::default();
        
        // The quit command should always succeed
        let result = run(&config, &args).await;
        assert!(result.is_ok());
    }
}
