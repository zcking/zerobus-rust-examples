# Zerobus Rust Examples

A collection of example applications demonstrating how to use the [Databricks Zerobus SDK for Rust](https://github.com/databricks/zerobus-sdk-rs).

## Overview

Zerobus is Databricks' streaming ingestion service that allows you to ingest data into Unity Catalog tables using Protocol Buffers over gRPC. This repository contains practical examples to help you get started.

For comprehensive documentation, see the official [Databricks Zerobus Ingest documentation](https://docs.databricks.com/aws/en/ingestion/lakeflow-connect/zerobus-ingest?language=Rust%C2%A0SDK).

## Examples

| Example | Description |
|---------|-------------|
| [Hello World](examples/hello-world/README.md) | Basic example demonstrating the fundamental workflow of the Zerobus SDK, including SDK initialization, stream creation, message encoding, record ingestion, and graceful shutdown. |
| [AWS Lambda SQS Ingestor](examples/aws-lambda-sqs-ingestor/README.md) | AWS Lambda function that processes SQS messages and ingests them into Unity Catalog tables via Zerobus. Includes Terraform infrastructure for deployment with SQS queue, Dead Letter Queue, and Lambda function configured for partial batch response. |
| [AWS Lambda Generic Ingestor](examples/aws-generic-ingestor/README.md) | Generic AWS Lambda function that can ingest events from any AWS service (API Gateway, EventBridge, S3, SNS, etc.) into Unity Catalog tables via Zerobus. Stores event payloads and Lambda context as JSON strings, making it suitable for centralized logging and event auditing. |

## Prerequisites

- Rust 1.70 or later
- A Databricks workspace with Zerobus enabled (contact your Databricks account representative if needed)
- Service principal with OAuth credentials (client ID and client secret)
- A Unity Catalog table configured for Zerobus ingestion
- [Buf](https://github.com/bufbuild/buf) or the Protocol Buffer compiler (`protoc`) for compiling protobuf bindings

## Setup

### 1. Clone this repository

```bash
git clone git@github.com:zcking/zerobus-rust-examples.git
cd zerobus-rust-examples
```

### 2. Configure environment variables

Copy the example environment file and fill in your actual values:

```bash
cp .env.example .env
```

Edit `.env` with your specific configuration and credentials.

Export the environment variables from `.env` in your shell:  

```bash
export $(grep -v '^#' .env | grep -v '^$' | xargs)
```

**Creating a service principal:**
1. In your Databricks workspace, go to **Settings** > **Identity and Access**
2. Create a new service principal and generate credentials
3. Grant the required permissions for your target table:
   ```sql
   GRANT USE CATALOG ON CATALOG <catalog> TO `<service-principal-uuid>`;
   GRANT USE SCHEMA ON SCHEMA <catalog.schema> TO `<service-principal-uuid>`;
   GRANT MODIFY, SELECT ON TABLE <catalog.schema.table> TO `<service-principal-uuid>`;
   ```

### 3. Install zerobus-generate tool

The `zerobus-generate` tool is used to generate Protocol Buffer schemas from Unity Catalog tables. Install it globally:

```bash
# Clone the Zerobus Rust SDK repository
git clone https://github.com/databricks/zerobus-sdk-rs.git
cd zerobus-sdk-rs/tools/generate_files

# Build the tool
cargo build --release

# Copy to your cargo bin directory (makes it available globally)
cp target/release/tools ~/.cargo/bin/zerobus-generate

# Verify installation
zerobus-generate --help
```

### 4. Generate Protocol Buffer schemas

Generate Protocol Buffer schemas from your Unity Catalog table using the `zerobus-generate` command:

```bash
# Set environment variables (or export from .env file)
export DATABRICKS_HOST="https://myworkspace.cloud.databricks.com"
export DATABRICKS_CLIENT_ID="your-client-id"
export DATABRICKS_CLIENT_SECRET="your-client-secret"
export TABLE_NAME="main.default.zerobus_hello_world"

# Generate .proto file, Rust code, and descriptor from the Unity Catalog table
zerobus-generate \
  --uc-endpoint $DATABRICKS_HOST \
  --client-id $DATABRICKS_CLIENT_ID \
  --client-secret $DATABRICKS_CLIENT_SECRET \
  --table $TABLE_NAME \
  --output-dir examples/hello-world/proto
```

This will generate three files in the proto directory:
- `<table_name>.proto` - Protocol Buffer schema definition
- `<table_name>.rs` - Rust code generated from the schema
- `<table_name>.descriptor` - Binary descriptor file for runtime use

**Note:** The proto directory structure is already set up in the examples. The `zerobus-generate` tool will create all necessary artifacts that match your Unity Catalog table schema. The `*.descriptor` files are not committed to Git because they are binary files.

## Project Structure

```
zerobus-rust-examples/
├── Cargo.toml              # Workspace configuration
├── README.md               # This file
├── .env.example            # Example environment configuration
└── examples/
    ├── hello-world/        # Basic hello world example
    ├── aws-lambda-sqs-ingestor/  # AWS Lambda SQS ingestor
    └── aws-generic-ingestor/  # AWS Lambda generic ingestor
```

Each example directory contains its own `README.md` with specific setup and usage instructions.

## Key Concepts

### SDK Initialization

The SDK requires two endpoints:
- **Zerobus Endpoint**: The gRPC endpoint for streaming data ingestion (format: `https://<workspace_id>.zerobus.<region>.cloud.databricks.com`)
- **Databricks Host**: Your workspace URL used for Unity Catalog authentication and table metadata

### Authentication

The SDK handles OAuth 2.0 authentication automatically using service principal credentials. You only need to provide:
- **Client ID**: Service principal application ID
- **Client Secret**: Service principal secret

These credentials are used to obtain and refresh access tokens as needed. The SDK manages token lifecycle internally.

### Stream Lifecycle

1. **Create Stream**: Opens an authenticated bidirectional gRPC stream to the Zerobus service
2. **Ingest Records**: Send Protocol Buffer encoded data representing table rows
3. **Acknowledgments**: Each record returns a future that resolves when the service acknowledges durability
4. **Flush**: Force pending records to be transmitted (useful before shutdown)
5. **Close**: Gracefully shutdown the stream, ensuring all records are acknowledged

### Protocol Buffers

The Zerobus service uses Protocol Buffers for efficient data serialization. Here's the workflow:

1. **Generate Schema**: Use the `zerobus-generate` tool (installed globally) to automatically generate Protocol Buffer definitions from your Unity Catalog table schema
2. **Include Generated Code**: The tool creates three files:
   - `.proto` - Protocol Buffer schema definition
   - `.rs` - Rust code with message structs
   - `.descriptor` - Binary descriptor for runtime type information
3. **Encode and Send**: Create instances of your message structs, encode them, and send via the stream

## Configuration Options

The SDK supports various configuration options via `StreamConfigurationOptions`:

| Option | Default | Description |
|--------|---------|-------------|
| `max_inflight_records` | 50000 | Maximum number of unacknowledged records |
| `recovery` | true | Enable automatic stream recovery on failures |
| `recovery_timeout_ms` | 15000 | Timeout for recovery operations (ms) |
| `recovery_backoff_ms` | 2000 | Delay between recovery attempts (ms) |
| `recovery_retries` | 3 | Maximum number of recovery attempts |
| `flush_timeout_ms` | 300000 | Timeout for flush operations (ms) |
| `server_lack_of_ack_timeout_ms` | 60000 | Server acknowledgment timeout (ms) |

## Troubleshooting

### Common Issues

**Authentication errors:**
- Verify your service principal credentials are correct
- Ensure the service principal has the required permissions on the target table
- Check that your Databricks host URL is correct

**Connection errors:**
- Verify your Zerobus endpoint format: `https://<workspace_id>.zerobus.<region>.cloud.databricks.com`
- Ensure network connectivity to your Databricks workspace
- Check that Zerobus is enabled for your workspace

**Schema errors:**
- Regenerate your Protocol Buffer files if your table schema has changed
- Ensure the descriptor file matches your current table schema
- Verify field types match between your data and the schema

## Resources

- [Databricks Zerobus Ingest Documentation](https://docs.databricks.com/aws/en/ingestion/lakeflow-connect/zerobus-ingest?language=Rust%C2%A0SDK)
- [Zerobus Rust SDK Repository](https://github.com/databricks/zerobus-sdk-rs)
- [Databricks Unity Catalog Documentation](https://docs.databricks.com/unity-catalog/)
- [Protocol Buffers Documentation](https://protobuf.dev/)

## License

Apache-2.0
