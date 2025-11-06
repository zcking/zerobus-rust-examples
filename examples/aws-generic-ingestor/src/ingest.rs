use anyhow::{Context, Result};
use databricks_zerobus_ingest_sdk::ZerobusStream;
use lambda_runtime::LambdaEvent;
use prost::Message;
use serde_json::Value;
use tracing::info;

use crate::proto::aws_raw_events::TableAwsRawEvents;

/// Ingest a Lambda event into Zerobus
pub async fn ingest_event(
    event: &LambdaEvent<Value>,
    stream: &mut ZerobusStream,
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

    // Extract request_id from context (minimal field)
    let request_id = event.context.request_id.clone();

    // Serialize payload as JSON string
    let payload_json = serde_json::to_string(&event.payload)
        .context("Failed to serialize event payload to JSON")?;

    // Serialize entire context as JSON string
    let context_json = serde_json::to_string(&event.context)
        .context("Failed to serialize Lambda context to JSON")?;

    // Extract deadline in milliseconds (cast from u64 to i64)
    let deadline = event.context.deadline as i64;

    // Create protobuf message
    let raw_event = TableAwsRawEvents {
        request_id: Some(request_id.clone()),
        payload: Some(payload_json),
        context: Some(context_json),
        deadline: Some(deadline),
        ingested_at: Some(ingested_at),
        ingested_date: Some(ingested_date),
    };

    // Encode and ingest
    let encoded = raw_event.encode_to_vec();
    let ack_future = stream.ingest_record(encoded).await?;
    ack_future.await?;

    info!("Successfully ingested event with request_id: {}", request_id);
    Ok(())
}

