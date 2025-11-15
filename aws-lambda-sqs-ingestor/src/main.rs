use anyhow::{Context, Result};
use aws_lambda_events::{
    event::sqs::{SqsBatchResponse, SqsEvent},
    sqs::{BatchItemFailure, SqsMessage, SqsMessageAttribute},
};
use base64::{engine::general_purpose, Engine as _};
use databricks_zerobus_ingest_sdk::{StreamConfigurationOptions, TableProperties, ZerobusSdk, ZerobusStream};
use lambda_runtime::{run, service_fn, Error, LambdaEvent};
use prost::bytes::Bytes;
use prost::Message;
use prost_types::DescriptorProto;
use std::sync::OnceLock;
use tracing::{error, info};

// Module for generated protobuf code
pub mod sqs_messages {
    include!("../gen/rust/sqs_messages.rs");
}
use crate::sqs_messages::TableSqsMessages;

// Global SDK instance for reuse across Lambda invocations
static SDK: OnceLock<ZerobusSdk> = OnceLock::new();

/// Initialize the Zerobus SDK (called once per Lambda container)
fn init_sdk() -> Result<&'static ZerobusSdk> {
    SDK.get_or_init(|| {
        let zerobus_endpoint = std::env::var("ZEROBUS_ENDPOINT")
            .expect("ZEROBUS_ENDPOINT environment variable must be set");
        let databricks_host = std::env::var("DATABRICKS_HOST")
            .expect("DATABRICKS_HOST environment variable must be set");

        ZerobusSdk::new(zerobus_endpoint, databricks_host)
            .expect("Failed to initialize ZerobusSdk")
    });
    Ok(SDK.get().expect("SDK should be initialized"))
}

/// Load the protobuf descriptor from the embedded descriptor file
fn load_descriptor_proto(file_name: &str, message_name: &str) -> DescriptorProto {
    const DESCRIPTOR_BYTES: &[u8] = include_bytes!("../gen/descriptors/sqs_messages.descriptor");

    let file_descriptor_set = prost_types::FileDescriptorSet::decode(DESCRIPTOR_BYTES)
        .expect("Failed to decode descriptor file");

    let file_descriptor_proto = file_descriptor_set
        .file
        .into_iter()
        .find(|f| f.name.as_ref().map(|n| n.as_str()) == Some(file_name))
        .expect("File descriptor not found");

    file_descriptor_proto
        .message_type
        .into_iter()
        .find(|m| m.name.as_ref().map(|n| n.as_str()) == Some(message_name))
        .expect("Message descriptor not found")
}

/// Convert SQS message attributes to protobuf message attributes structure
fn convert_message_attributes(
    attrs: &std::collections::HashMap<String, SqsMessageAttribute>,
) -> std::collections::HashMap<String, crate::sqs_messages::table_sqs_messages::MessageAttributes> {
    let mut result = std::collections::HashMap::new();

    for (key, attr) in attrs {
        let binary_value = attr.binary_value.as_ref().map(|bv| {
            // Base64Data might be a newtype wrapper - try Debug format or direct access
            let b64_str = format!("{:?}", bv);
            // Remove quotes if Debug adds them
            let b64_str = b64_str.trim_matches('"');
            Bytes::from(general_purpose::STANDARD.decode(b64_str).unwrap_or_default())
        });

        let binary_list_values: Vec<Bytes> = attr.binary_list_values
            .iter()
            .map(|bv| {
                let b64_str = format!("{:?}", bv);
                let b64_str = b64_str.trim_matches('"');
                Bytes::from(general_purpose::STANDARD.decode(b64_str).unwrap_or_default())
            })
            .collect();

        let message_attr = crate::sqs_messages::table_sqs_messages::MessageAttributes {
            string_value: attr.string_value.clone(),
            binary_value,
            string_list_values: attr.string_list_values.clone(),
            binary_list_values,
            data_type: attr.data_type.clone(),
        };
        result.insert(key.clone(), message_attr);
    }

    result
}

/// Convert SQS message attributes (system attributes) to protobuf map
fn convert_attributes(
    attrs: &std::collections::HashMap<String, String>,
) -> std::collections::HashMap<String, String> {
    attrs.clone()
}

/// Process a single SQS message and ingest it into Zerobus
async fn process_message(
    message: &SqsMessage,
    stream: &mut ZerobusStream,
    aws_region: &str,
    event_source_arn: &str,
) -> Result<()> {
    // Get current timestamp in microseconds
    let now = std::time::SystemTime::now();
    let ingested_at = now
        .duration_since(std::time::UNIX_EPOCH)
        .context("Failed to get system time")?
        .as_micros() as i64;
    // Get the current date as int32 (days since Unix epoch) 
    let ingested_date = now
        .duration_since(std::time::UNIX_EPOCH)
        .context("Failed to get system time")?
        .as_secs() as i32 / 86400;

    // Extract message fields
    let message_id = message
        .message_id
        .as_ref()
        .context("Message ID is required")?
        .clone();
    let message_id_for_log = message_id.clone();
    let receipt_handle = message
        .receipt_handle
        .as_ref()
        .context("Receipt handle is required")?
        .clone();
    let body = message.body.as_deref().unwrap_or_default().to_string();
    let md5_of_body = message.md5_of_body.as_deref().unwrap_or_default().to_string();
    let md5_of_message_attributes = message
        .md5_of_message_attributes
        .as_deref()
        .unwrap_or_default()
        .to_string();

    // Convert attributes
    let attributes = convert_attributes(&message.attributes);
    let message_attributes = convert_message_attributes(&message.message_attributes);

    // Create protobuf message
    let sqs_message = TableSqsMessages {
        message_id: Some(message_id),
        receipt_handle: Some(receipt_handle),
        body: Some(body),
        md5_of_body: Some(md5_of_body),
        md5_of_message_attributes: Some(md5_of_message_attributes),
        attributes,
        message_attributes,
        queue_arn: Some(event_source_arn.to_string()),
        aws_region: Some(aws_region.to_string()),
        ingested_at: Some(ingested_at),
        ingested_date: Some(ingested_date),
    };

    // Encode and ingest
    let encoded = sqs_message.encode_to_vec();
    let ack_future = stream.ingest_record(encoded).await?;
    ack_future.await?;

    info!("Successfully ingested message: {}", message_id_for_log);
    Ok(())
}

/// Lambda handler function
async fn function_handler(event: LambdaEvent<SqsEvent>) -> Result<SqsBatchResponse, Error> {
    let sdk = init_sdk().map_err(|e| Error::from(format!("Failed to initialize SDK: {}", e)))?;

    let table_name = std::env::var("TABLE_NAME")
        .map_err(|_| Error::from("TABLE_NAME environment variable must be set"))?;
    let client_id = std::env::var("DATABRICKS_CLIENT_ID")
        .map_err(|_| Error::from("DATABRICKS_CLIENT_ID environment variable must be set"))?;
    let client_secret = std::env::var("DATABRICKS_CLIENT_SECRET")
        .map_err(|_| Error::from("DATABRICKS_CLIENT_SECRET environment variable must be set"))?;

    // Load descriptor
    let descriptor_proto = load_descriptor_proto("sqs_messages.proto", "table_sqs_messages");

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

    // Extract AWS region and event source ARN from first record (all records from same queue)
    let (event_source_arn, aws_region) = event
        .payload
        .records
        .first()
        .and_then(|r| Some((r.event_source_arn.as_ref().cloned().unwrap_or_default(), r.aws_region.as_ref().cloned().unwrap_or_default())))
        .unwrap_or_default();

    let mut batch_item_failures = Vec::new();

    // Process each message
    for record in event.payload.records {
        let message_id = record.message_id.clone().unwrap_or_default();

        match process_message(&record, &mut stream, &aws_region, &event_source_arn).await {
            Ok(_) => {
                info!("Successfully processed message: {}", message_id);
            }
            Err(e) => {
                error!("Failed to process message {}: {}", message_id, e);
                batch_item_failures.push(BatchItemFailure {
                    item_identifier: message_id,
                });
            }
        }
    }

    // Flush all pending writes and close the stream
    if let Err(e) = stream.close().await {
        error!("Failed to close stream: {}", e);
        
        // TODO: check e.is_retryable and retry where possible

        // TODO: use strema.get_unacked_records() so we can push unacknowledged records to a DLQ
        let unacked = stream.get_unacked_records().await?;
        println!("Failed to acknowledge {} records", unacked.len()); // TODO: switch to logging
        
        // Recreates the stream with the same configuration and automatically re-ingests all records that weren't acknowledged.
        sdk.recreate_stream(stream).await?;
    }

    Ok(SqsBatchResponse {
        batch_item_failures,
    })
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Install the default CryptoProvider early in your application
    rustls::crypto::aws_lc_rs::default_provider().install_default().unwrap();

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_target(false)
        .init();

    run(service_fn(function_handler)).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use lambda_runtime::{Context, LambdaEvent};

    #[tokio::test]
    async fn test_event_handler() {
        let event = LambdaEvent::new(SqsEvent::default(), Context::default());
        let response = function_handler(event).await.unwrap();
        assert_eq!(SqsBatchResponse::default(), response);
    }
}
