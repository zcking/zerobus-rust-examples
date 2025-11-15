use databricks_zerobus_ingest_sdk::{StreamConfigurationOptions, TableProperties};
use lambda_runtime::{Error, LambdaEvent};
use serde_json::Value;
use tracing::{error, info};

use crate::ingest::ingest_event;
use crate::proto::load_descriptor_proto;
use crate::sdk::init_sdk;

/// Lambda handler function
pub async fn function_handler(event: LambdaEvent<Value>) -> Result<String, Error> {
    let sdk = init_sdk().map_err(|e| Error::from(format!("Failed to initialize SDK: {}", e)))?;

    let table_name = std::env::var("TABLE_NAME")
        .map_err(|_| Error::from("TABLE_NAME environment variable must be set"))?;
    let client_id = std::env::var("DATABRICKS_CLIENT_ID")
        .map_err(|_| Error::from("DATABRICKS_CLIENT_ID environment variable must be set"))?;
    let client_secret = std::env::var("DATABRICKS_CLIENT_SECRET")
        .map_err(|_| Error::from("DATABRICKS_CLIENT_SECRET environment variable must be set"))?;

    // Load descriptor
    let descriptor_proto = load_descriptor_proto("aws_raw_events.proto", "table_aws_raw_events");

    // Configure table properties
    let table_properties = TableProperties {
        table_name: table_name.clone(),
        descriptor_proto,
    };

    // Configure stream options
    let stream_options = StreamConfigurationOptions {
        max_inflight_records: 1000,
        ..Default::default()
    };

    // Create stream
    let mut stream = sdk
        .create_stream(table_properties, client_id, client_secret, Some(stream_options))
        .await
        .map_err(|e| Error::from(format!("Failed to create stream: {}", e)))?;

    info!("Processing event with request_id: {}", event.context.request_id);

    // Ingest the event
    match ingest_event(&event, &mut stream).await {
        Ok(_) => {
            info!("Successfully processed event");
        }
        Err(e) => {
            error!("Failed to process event: {}", e);
            return Err(Error::from(format!("Failed to ingest event: {}", e)));
        }
    }

    // Flush all pending writes and close the stream
    if let Err(e) = stream.close().await {
        error!("Failed to close stream: {}", e);

        // Get unacknowledged records for potential retry
        let unacked = stream.get_unacked_records().await.map_err(|e| {
            Error::from(format!("Failed to get unacked records: {}", e))
        })?;
        
        if !unacked.is_empty() {
            error!("Failed to acknowledge {} records", unacked.len());
            // Recreate the stream with the same configuration and automatically re-ingest all records that weren't acknowledged.
            sdk.recreate_stream(stream).await.map_err(|e| {
                Error::from(format!("Failed to recreate stream: {}", e))
            })?;
        }
        
        return Err(Error::from(format!("Failed to close stream: {}", e)));
    }

    Ok("Success".to_string())
}

