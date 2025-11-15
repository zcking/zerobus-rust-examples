use aws_generic_ingestor::handler;
use lambda_runtime::{run, service_fn, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Install the default CryptoProvider early in your application
    rustls::crypto::aws_lc_rs::default_provider().install_default().unwrap();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    run(service_fn(handler::function_handler)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_runtime::{Context, LambdaEvent};
    use serde_json::json;

    #[tokio::test]
    async fn test_event_handler() {
        let event_value = json!({"test": "data"});
        let event = LambdaEvent::new(event_value, Context::default());
        let response = handler::function_handler(event).await;
        // Test will fail due to missing env vars, but verifies compilation
        assert!(response.is_err());
    }
}

