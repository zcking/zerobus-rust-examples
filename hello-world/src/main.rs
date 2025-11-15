use anyhow::Result;
use databricks_zerobus_ingest_sdk::{ZerobusSdk, TableProperties, StreamConfigurationOptions};
use prost::Message;
use prost_types::DescriptorProto;

// Example protobuf message - in a real application, this would be generated
// from your Unity Catalog table schema using the zerobus CLI tool
// #[derive(Clone, PartialEq, Message)]
// pub struct HelloMessage {
//     #[prost(string, tag = "1")]
//     pub msg: String,
//     #[prost(int64, tag = "2")]
//     pub timestamp: i64,
// }

pub mod hello_world {
    include!("../gen/rust/zerobus_hello_world.rs");
} // Module name is arbitrary. Change to match your module name.
use crate::hello_world::TableZerobusHelloWorld;

#[tokio::main]
async fn main() -> Result<()> {
    println!("Zerobus Hello World Example");
    println!("=============================\n");

    // Configuration - in a real application, these would come from environment variables
    // or configuration files
    let zerobus_endpoint = std::env::var("ZEROBUS_ENDPOINT")
        .expect("ZEROBUS_ENDPOINT environment variable must be set");

    let databricks_host = std::env::var("DATABRICKS_HOST")
        .expect("DATABRICKS_HOST environment variable must be set");

    let client_id = std::env::var("DATABRICKS_CLIENT_ID")
        .expect("DATABRICKS_CLIENT_ID environment variable must be set");

    let client_secret = std::env::var("DATABRICKS_CLIENT_SECRET")
        .expect("DATABRICKS_CLIENT_SECRET environment variable must be set");

    let table_name = std::env::var("TABLE_NAME")
        .expect("TABLE_NAME environment variable must be set");
    
    let descriptor_proto = load_descriptor_proto(
        "zerobus_hello_world.proto",
        "table_zerobus_hello_world"
    );

    println!("Initializing Zerobus SDK...");

    // Step 1: Initialize the SDK
    let sdk = ZerobusSdk::new(
        zerobus_endpoint.clone(),
        databricks_host.clone(),
    )?;

    println!("Creating stream to table: {}", table_name);

    // Step 2: Configure the table properties
    let table_properties = TableProperties {
        table_name: table_name.clone(),
        // In a real application, you would load the actual protobuf descriptor
        // generated from your Unity Catalog table schema
        descriptor_proto: descriptor_proto,
    };

    // Step 3: Configure stream options
    let stream_options = StreamConfigurationOptions {
        max_inflight_records: 1000,
        ..Default::default()
    };

    // Step 4: Create a stream with OAuth credentials
    let mut stream = sdk.create_stream(
        table_properties,
        client_id,
        client_secret,
        Some(stream_options),
    ).await?;

    println!("Stream created successfully!");

    // Step 5: Create and encode a hello world message
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)?
        .as_micros() as i64; // Convert to microseconds
    let hello_msg = TableZerobusHelloWorld {
        msg: Some("Hello, Zerobus!".to_string()),
        ingested_at: Some(now),
    };

    println!("\nSending message: {}", hello_msg.msg.as_ref().unwrap());

    // Step 6: Encode the message using Protocol Buffers
    let encoded = hello_msg.encode_to_vec();

    // Step 7: Ingest the record and get an acknowledgment future
    let ack_future = stream.ingest_record(encoded).await?;

    println!("Message sent, waiting for acknowledgment...");

    // Step 8: Wait for acknowledgment
    ack_future.await?;

    println!("Message acknowledged successfully!");

    // Step 9: Flush any pending records
    stream.flush().await?;

    println!("Stream flushed.");

    // Step 10: Close the stream gracefully
    stream.close().await?;

    println!("\nStream closed. Hello World example complete!");

    Ok(())
}

fn load_descriptor_proto(
    file_name: &str,
    message_name: &str
) -> DescriptorProto {
    // Embed the descriptor file at compile time
    const DESCRIPTOR_BYTES: &[u8] = include_bytes!("../gen/descriptors/zerobus_hello_world.descriptor");
    
    let file_descriptor_set = prost_types::FileDescriptorSet::decode(
        DESCRIPTOR_BYTES
    ).unwrap();

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