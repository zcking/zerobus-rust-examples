# AWS Lambda Generic Ingestor

A Rust-based AWS Lambda function that processes **any** Lambda event type and ingests it into a Unity Catalog table using the Databricks Zerobus SDK.

## Overview

This example demonstrates how to:
- Process **any** Lambda event type (API Gateway, EventBridge, S3, SNS, SQS, etc.)
- Serialize event payloads and Lambda context as JSON strings
- Ingest generic events into Unity Catalog tables via Zerobus
- Deploy infrastructure using Terraform

Unlike the SQS-specific ingestor, this function is designed to work with any Lambda event source, making it a versatile solution for ingesting AWS events into Databricks.

## Prerequisites

- Rust 1.70 or later
- [buf](https://buf.build) CLI tool: `brew install bufbuild/buf/buf`
- `zerobus-generate` tool (see [root README](../../README.md) for installation)
- [cargo-lambda](https://github.com/cargo-lambda/cargo-lambda): `brew install cargo-lambda`
- Terraform >= 1.0
- AWS CLI configured with appropriate credentials
- Databricks workspace with Zerobus enabled, service principal credentials, and Unity Catalog table

## Setup

See the [root README](../../README.md) for initial workspace setup (service principal creation, environment variables, etc.).

### 1. Create Unity Catalog Table

First, create the target table in Unity Catalog using the following SQL:

```sql
CREATE OR REPLACE TABLE aws_raw_events (
  request_id STRING COMMENT 'AWS Lambda request ID from the execution context',
  
  payload STRING COMMENT 'Full event payload serialized as JSON string. This contains the original event data from the Lambda trigger (e.g., API Gateway request, S3 event, EventBridge event, etc.)',
  
  context STRING COMMENT 'Full Lambda execution context serialized as JSON string. Includes request_id, deadline, invoked_function_arn, xray_trace_id, and other context metadata',
  
  deadline BIGINT COMMENT 'Execution deadline for the Lambda invocation in milliseconds since Unix epoch',
  
  ingested_at TIMESTAMP COMMENT 'The timestamp when the event was ingested into this table (microseconds since Unix epoch)',
  
  ingested_date DATE COMMENT 'The date when the event was ingested into this table (for partitioning)'
)
USING DELTA
TBLPROPERTIES (
    delta.enableRowTracking = false
)
COMMENT 'Generic AWS Lambda events ingested from any event source (API Gateway, EventBridge, S3, SNS, SQS, etc.)'
;
```

Grant permissions to your service principal:

```sql
GRANT USE CATALOG ON CATALOG <catalog> TO `<service-principal-uuid>`;
GRANT USE SCHEMA ON SCHEMA <catalog.schema> TO `<service-principal-uuid>`;
GRANT MODIFY, SELECT ON TABLE <catalog.schema.aws_raw_events> TO `<service-principal-uuid>`;
```

### 2. Generate and Compile Protocol Buffers

```bash
cd examples/aws-generic-ingestor

# Generate .proto file from Unity Catalog table
make proto-generate

# Compile .proto to Rust bindings and descriptors
make proto-compile

# Or run both steps together:
make proto
```

This creates:
- `proto/aws_raw_events.proto` - Source schema (committed to git)
- `gen/rust/aws_raw_events.rs` - Rust message structs (generated)
- `gen/descriptors/aws_raw_events.descriptor` - Runtime descriptor (generated)

### 3. Build and Package

Build the Lambda function (automatically compiles protos first):

```bash
# Development build
make build

# Production build
make build ARGS='--arm64 --release'
```

This prepares a `../../target/lambda/aws-generic-ingestor/bootstrap.zip` file that is ready for deployment.

## Local Testing

In one terminal, run `make serve` to start running a local emulator of the Lambda function that you can invoke for testing.

In another terminal, run `make invoke` to send a test event to the lambda emulator. You can test with different event types:

```bash
# Test with API Gateway event
make invoke ARGS='--data-example apigw-request'

# Test with S3 event
make invoke ARGS='--data-example s3-event'

# Test with custom JSON from file
make invoke ARGS='--data-file path/to/event.json'
```

### Verify Data

Query your Unity Catalog table:

```sql
SELECT
  parse_json(payload) as parsed_payload,
  *
FROM aws_raw_events
ORDER BY ingested_at DESC
LIMIT 10;
```

## Deployment

See the [Terraform README](terraform/README.md) for detailed deployment instructions.

### Quick Start

1. Create `terraform/terraform.tfvars`:

```hcl
aws_region = "us-west-2"
function_name = "zerobus-generic-ingestor"

databricks_host = "https://myworkspace.cloud.databricks.com"
databricks_client_id = "your-client-id"
databricks_client_secret = "your-client-secret"
zerobus_endpoint = "https://<workspace_id>.zerobus.<region>.cloud.databricks.com"

table_name = "zach_king.zerobus.aws_raw_events"
```

2. Deploy:

```bash
cd terraform
terraform init
terraform plan
terraform apply
```

### Connecting Event Sources

After deploying the Lambda function, you can connect it to any AWS event source:

- **API Gateway**: Create an API Gateway REST or HTTP API and configure it to invoke the Lambda
- **EventBridge**: Create an EventBridge rule that targets the Lambda function
- **S3**: Configure S3 bucket notifications to trigger the Lambda
- **SNS**: Subscribe the Lambda function to an SNS topic
- **SQS**: Create an SQS event source mapping (see aws-lambda-sqs-ingestor for SQS-specific handling)
- **CloudWatch Events/Logs**: Create a subscription filter or log group subscription

See the [Terraform README](terraform/README.md) for more details on connecting event sources.

## Testing

### Send Test Event

After deployment, you can test the Lambda function directly using the AWS CLI:

```bash
# Get function name
FUNCTION_NAME=$(cd terraform && terraform output -raw lambda_function_name)

# Invoke with a simple test event
aws lambda invoke \
  --function-name "$FUNCTION_NAME" \
  --payload '{"key": "value"}' \
  response.json

# View response
cat response.json
```

### View Logs

Monitor CloudWatch logs:

```bash
LOG_GROUP=$(cd terraform && terraform output -raw cloudwatch_log_group_name)
aws logs tail "$LOG_GROUP" --follow
```

## Architecture

### Components

- **Lambda Function**: Processes any Lambda event and ingests it via Zerobus
- **CloudWatch Logs**: Logs function execution and errors
- **Event Sources**: Any AWS service that can trigger Lambda (API Gateway, EventBridge, S3, SNS, etc.)

### Event Processing

The Lambda function:
1. Receives the event from any AWS event source
2. Serializes the event payload as a JSON string
3. Serializes the Lambda execution context as a JSON string
4. Extracts the request_id (minimal context field)
5. Extracts the deadline (execution deadline in milliseconds)
6. Calculates ingestion timestamp and date
7. Ingests the record into Unity Catalog via Zerobus

### Error Handling

- Errors during ingestion are logged to CloudWatch
- Stream recreation is attempted if the stream fails to close
- Unacknowledged records are automatically re-ingested on stream recreation

## Configuration

### Environment Variables

The Lambda function requires these environment variables (set via Terraform):

- `DATABRICKS_HOST` - Databricks workspace URL
- `DATABRICKS_CLIENT_ID` - Service principal client ID
- `DATABRICKS_CLIENT_SECRET` - Service principal secret
- `ZEROBUS_ENDPOINT` - Zerobus gRPC endpoint
- `TABLE_NAME` - Unity Catalog table name (e.g., `zach_king.zerobus.aws_raw_events`)
- `AWS_REGION` - AWS region (auto-set by Lambda runtime)

### Lambda Configuration

Default configuration (configurable via Terraform):

- **Architecture**: ARM64
- **Memory**: 512 MB
- **Timeout**: 60 seconds

## Use Cases

This generic ingestor is useful for:

- **Centralized Logging**: Ingest all Lambda events into a single Delta table for analysis
- **Event Auditing**: Track all events across different AWS services in one place
- **Data Lake Ingestion**: Stream events from multiple sources into Databricks
- **Multi-Service Integration**: Handle events from API Gateway, EventBridge, S3, SNS, etc. with a single function

## Troubleshooting

### Lambda Function Not Receiving Events

1. Verify the event source is configured correctly (API Gateway integration, EventBridge rule, S3 notification, etc.)
2. Check IAM permissions for the event source
3. Verify the Lambda function ARN matches what's configured in the event source

### Events Not Being Ingested

1. Check CloudWatch logs for errors:
   ```bash
   LOG_GROUP=$(cd terraform && terraform output -raw cloudwatch_log_group_name)
   aws logs tail "$LOG_GROUP" --follow
   ```
2. Verify Databricks credentials are correct
3. Check table schema matches expected format
4. Ensure proto files are generated and descriptor file is present

### Build Issues

- Ensure `cargo-lambda` is installed: `cargo install cargo-lambda`
- Verify Rust version: `rustc --version` (should be 1.70+)
- Clean and rebuild: `make clean build`

## Code Structure

This example uses a modular code structure:

- `src/main.rs` - Entry point, initializes tracing and runs Lambda runtime
- `src/handler.rs` - Lambda handler function that orchestrates the ingestion flow
- `src/sdk.rs` - SDK initialization and management
- `src/proto.rs` - Protocol buffer utilities and descriptor loading
- `src/ingest.rs` - Event ingestion logic that serializes and encodes events

## Resources

- [Databricks Zerobus Documentation](https://docs.databricks.com/aws/en/ingestion/lakeflow-connect/zerobus-ingest?language=Rust%20SDK)
- [AWS Lambda Rust Runtime](https://github.com/awslabs/aws-lambda-rust-runtime)
- [AWS Lambda Event Sources](https://docs.aws.amazon.com/lambda/latest/dg/lambda-services.html)
- [Terraform AWS Provider](https://registry.terraform.io/providers/hashicorp/aws/latest/docs)

