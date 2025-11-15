# AWS Lambda SQS Ingestor

A Rust-based AWS Lambda function that processes SQS messages and ingests them into a Unity Catalog table using the Databricks Zerobus SDK.

## Overview

This example demonstrates how to:
- Process SQS messages in an AWS Lambda function
- Convert SQS message data to Protocol Buffer format
- Ingest messages into Unity Catalog tables via Zerobus
- Handle partial batch failures with retry support
- Deploy infrastructure using Terraform

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
CREATE OR REPLACE TABLE sqs_messages (
  message_id STRING COMMENT 'Each message receives a system-assigned message ID. This identifier is useful for identifying messages. The maximum length of a message ID is 100 characters.',

  receipt_handle STRING COMMENT 'Every time you receive a message from a queue, you receive a receipt handle for that message. This handle is associated with the action of receiving the message, not with the message itself. To delete the message or to change the message visibility, you must provide the receipt handle (not the message ID). Thus, you must always receive a message before you can delete it (you can\'t put a message into the queue and then recall it). The maximum length of a receipt handle is 1,024 characters.',

  body STRING COMMENT 'The message body that was sent. The minimum size is one character. The maximum size is 1 MiB or 1,048,576 bytes',

  md5_of_body STRING COMMENT 'MD5 checksum of the message body',
  md5_of_message_attributes STRING COMMENT 'MD5 checksum of the message attributes',

  attributes MAP<STRING, STRING> COMMENT 'The message system attribute to send. Each message system attribute consists of a Name, Type, and Value.',

  message_attributes MAP<STRING, STRUCT<
    string_value: STRING,
    binary_value: BINARY,
    string_list_values: ARRAY<STRING>,
    binary_list_values: ARRAY<BINARY>,
    data_type: STRING
  >> COMMENT 'Each message attribute consists of a Name, Type, and Value. For more information, see Amazon SQS message attributes in the [Amazon SQS Developer Guide](https://docs.aws.amazon.com/AWSSimpleQueueService/latest/SQSDeveloperGuide/sqs-message-metadata.html#sqs-message-attributes).',

  queue_arn STRING COMMENT 'The Amazon Resource Name (ARN) of the queue from which the message was sent',

  aws_region STRING COMMENT 'The AWS region in which the queue is located',

  ingested_at TIMESTAMP COMMENT 'The timestamp when the message was ingested into this table',
  ingested_date DATE COMMENT 'The date when the message was ingested into this table.'
)
TBLPROPERTIES (delta.enableRowTracking = false)
COMMENT 'Messages ingested from SQS.'
;
```

Grant permissions to your service principal:

```sql
GRANT USE CATALOG ON CATALOG <catalog> TO `<service-principal-uuid>`;
GRANT USE SCHEMA ON SCHEMA <catalog.schema> TO `<service-principal-uuid>`;
GRANT MODIFY, SELECT ON TABLE <catalog.schema.table> TO `<service-principal-uuid>`;
```

### 2. Generate and Compile Protocol Buffers

```bash
cd examples/aws-lambda-sqs-ingestor

# Generate .proto file from Unity Catalog table
make proto-generate

# Compile .proto to Rust bindings and descriptors
make proto-compile

# Or run both steps together:
make proto
```

This creates:
- `proto/sqs_messages.proto` - Source schema (committed to git)
- `gen/rust/sqs_messages.rs` - Rust message structs (generated)
- `gen/descriptors/sqs_messages.descriptor` - Runtime descriptor (generated)

### 3. Build and Package

Build the Lambda function (automatically compiles protos first):

```bash
# Development build
make build

# Production build
make build ARGS='--arm64 --release'
```

This prepares a `../../target/lambda/aws-lambda-sqs-ingestor/bootstrap.zip` file that is ready for deployment.

## Local Testing

In one terminal, run `make serve` to start running a local emulator of the Lambda function that you can invoke for testing.

In another terminal, run `make invoke` to send a test SQS event to the lambda emulator.

Run `make invoke ARGS=-h` to see more options for overriding arguments to the invoke helper. For example, to pass your own event payload from a file:  

```bash
make invoke ARGS='--data-file path/to/data.json'
```

## Deployment

See the [Terraform README](terraform/README.md) for detailed deployment instructions.

### Quick Start

1. Create `terraform/terraform.tfvars`:

```hcl
aws_region = "us-west-2"
function_name = "zerobus-sqs-ingestor"
queue_name = "zerobus-ingestion-queue"

databricks_host = "https://myworkspace.cloud.databricks.com"
databricks_client_id = "your-client-id"
databricks_client_secret = "your-client-secret"
zerobus_endpoint = "https://<workspace_id>.zerobus.<region>.cloud.databricks.com"

table_name = "zach_king.zerobus.sqs_messages"
```

2. Deploy:

```bash
cd terraform
terraform init
terraform plan
terraform apply
```

## Testing

### Send Test Message

After deployment, send a test message to the SQS queue:

```bash
# Get queue URL
QUEUE_URL=$(cd terraform && terraform output -raw sqs_queue_url)

# Send simple message
aws sqs send-message \
  --queue-url "$QUEUE_URL" \
  --message-body "Test message from AWS CLI"

# Send message with attributes
aws sqs send-message \
  --queue-url "$QUEUE_URL" \
  --message-body "Test message with attributes" \
  --message-attributes '{
    "test-attr": {
      "DataType": "String",
      "StringValue": "test-value"
    },
    "binary-attr": {
      "DataType": "Binary",
      "BinaryValue": "SGVsbG8gV29ybGQ="
    }
  }'
```

### View Logs

Monitor CloudWatch logs:

```bash
LOG_GROUP=$(cd terraform && terraform output -raw cloudwatch_log_group_name)
aws logs tail "$LOG_GROUP" --follow
```

### Verify Data

Query your Unity Catalog table:

```sql
SELECT * FROM sqs_messages
ORDER BY ingested_at DESC
LIMIT 10;
```

## Architecture

### Components

- **SQS Queue**: Receives messages for processing
- **Dead Letter Queue (DLQ)**: Receives messages that fail after max retry attempts
- **Lambda Function**: Processes SQS messages and ingests them via Zerobus
- **CloudWatch Logs**: Logs function execution and errors
- **Event Source Mapping**: Connects SQS queue to Lambda function with partial batch response support

### Partial Batch Response

The Lambda function implements partial batch response, which means:
- If some messages in a batch fail, only those messages are retried
- Successful messages are not reprocessed
- Failed message IDs are returned in the batch response
- Lambda retries only the failed messages

### Error Handling

- Messages that fail processing are tracked in `batch_item_failures`
- After `dlq_max_receive_count` (default: 3) failed attempts, messages are sent to the DLQ
- Lambda logs all errors to CloudWatch for debugging

## Configuration

### Environment Variables

The Lambda function requires these environment variables (set via Terraform):

- `DATABRICKS_HOST` - Databricks workspace URL
- `DATABRICKS_CLIENT_ID` - Service principal client ID
- `DATABRICKS_CLIENT_SECRET` - Service principal secret
- `ZEROBUS_ENDPOINT` - Zerobus gRPC endpoint
- `TABLE_NAME` - Unity Catalog table name (e.g., `zach_king.zerobus.sqs_messages`)
- `AWS_REGION` - AWS region (auto-set by Lambda runtime)

### Lambda Configuration

Default configuration (configurable via Terraform):

- **Architecture**: ARM64
- **Memory**: 512 MB
- **Timeout**: 60 seconds
- **Batch Size**: 10 messages
- **Visibility Timeout**: 300 seconds

### SQS Configuration

- **Message Retention**: 4 days
- **DLQ Retention**: 14 days
- **Max Receive Count**: 3 (before DLQ)

## Troubleshooting

### Lambda Function Not Processing Messages

1. Check event source mapping status:
   ```bash
   aws lambda list-event-source-mappings --function-name zerobus-sqs-ingestor
   ```

2. Verify IAM permissions
3. Check CloudWatch logs for errors

### Messages Going to DLQ

1. Review Lambda logs to identify failure reasons
2. Check DLQ for failed messages:
   ```bash
   DLQ_URL=$(cd terraform && terraform output -raw dlq_url)
   aws sqs receive-message --queue-url "$DLQ_URL"
   ```

3. Common issues:
   - Authentication errors (check Databricks credentials)
   - Table schema mismatches (regenerate proto files)
   - Network connectivity issues

### Build Issues

- Ensure `cargo-lambda` is installed: `cargo install cargo-lambda`
- Verify Rust version: `rustc --version` (should be 1.70+)
- Clean and rebuild: `make clean build`

## Resources

- [Databricks Zerobus Documentation](https://docs.databricks.com/aws/en/ingestion/lakeflow-connect/zerobus-ingest?language=Rust%20SDK)
- [AWS Lambda Rust Runtime](https://github.com/awslabs/aws-lambda-rust-runtime)
- [AWS SQS Documentation](https://docs.aws.amazon.com/sqs/)
- [Terraform AWS Provider](https://registry.terraform.io/providers/hashicorp/aws/latest/docs)

