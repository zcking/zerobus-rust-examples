# Terraform Deployment Guide

This directory contains Terraform configuration for deploying the AWS Lambda SQS Ingestor infrastructure.

## Prerequisites

- [Terraform](https://www.terraform.io/downloads) >= 1.0
- [AWS CLI](https://aws.amazon.com/cli/) configured with appropriate credentials
- Built Lambda function binary (see main README for build instructions)
- AWS account with permissions to create:
  - Lambda functions
  - SQS queues
  - IAM roles and policies
  - CloudWatch log groups

## Build and Package Lambda Function

Before deploying with Terraform, you must build and package the Lambda function:

```bash
make build
```

## Configuration

### Create terraform.tfvars

Create a `terraform.tfvars` file with your configuration:

```hcl
aws_region = "us-west-2"
function_name = "zerobus-sqs-ingestor"
queue_name = "zerobus-ingestion-queue"

# Databricks configuration
databricks_host = "https://myworkspace.cloud.databricks.com"
databricks_client_id = "your-client-id"
databricks_client_secret = "your-client-secret"
zerobus_endpoint = "https://<workspace_id>.zerobus.<region>.cloud.databricks.com"

# Table configuration
table_name = "zach_king.zerobus.sqs_messages"

# Lambda configuration
lambda_zip_path = "../../target/lambda/aws-lambda-sqs-ingestor/bootstrap.zip"
memory_size = 512
timeout = 60

# SQS configuration
batch_size = 10
visibility_timeout_seconds = 300
dlq_max_receive_count = 3
```

**Important:** Never commit `terraform.tfvars` to version control as it contains sensitive credentials.

## Deployment

### Initialize Terraform

```bash
cd terraform
terraform init
```

### Review Changes

```bash
terraform plan
```

### Apply Configuration

```bash
terraform apply
```

You will be prompted to confirm. Type `yes` to proceed.

## Testing

After deployment, send a test message to the SQS queue:

```bash
# Get the queue URL
QUEUE_URL=$(terraform output -raw sqs_queue_url)

# Send a test message
aws sqs send-message \
  --queue-url "$QUEUE_URL" \
  --message-body "Test message from AWS CLI"

# Send a message with attributes
aws sqs send-message \
  --queue-url "$QUEUE_URL" \
  --message-body "Test message with attributes" \
  --message-attributes '{
    "test-attr": {
      "DataType": "String",
      "StringValue": "test-value"
    }
  }'
```

### View Logs

Monitor CloudWatch logs to verify message processing:

```bash
# Get log group name
LOG_GROUP=$(terraform output -raw cloudwatch_log_group_name)

# Tail logs
aws logs tail "$LOG_GROUP" --follow

# View recent logs
aws logs tail "$LOG_GROUP" --since 5m
```

### Verify Data in Unity Catalog

Query your Unity Catalog table to verify messages were ingested:

```sql
SELECT * FROM zach_king.zerobus.sqs_messages
ORDER BY ingested_at DESC
LIMIT 10;
```

## Cleanup

To destroy all resources:

```bash
terraform destroy
```

**Warning:** This will delete the SQS queues and all messages in them. Ensure you've processed all important messages before destroying.

## Troubleshooting

### Lambda Function Not Receiving Messages

1. Check the event source mapping:
   ```bash
   aws lambda get-event-source-mapping --uuid <mapping-uuid>
   ```

2. Verify the Lambda function has permission to read from SQS:
   ```bash
   aws iam get-role-policy --role-name <role-name> --policy-name <policy-name>
   ```

3. Check CloudWatch logs for errors

### Messages Going to DLQ

1. Check DLQ for failed messages:
   ```bash
   DLQ_URL=$(terraform output -raw dlq_url)
   aws sqs receive-message --queue-url "$DLQ_URL"
   ```

2. Review Lambda logs to identify the failure reason

3. Adjust `dlq_max_receive_count` if needed (default is 3)

### Authentication Errors

- Verify Databricks credentials are correct in `terraform.tfvars`
- Ensure service principal has permissions on the target table:
  ```sql
  GRANT USE CATALOG ON CATALOG <catalog> TO `<service-principal-uuid>`;
  GRANT USE SCHEMA ON SCHEMA <catalog.schema> TO `<service-principal-uuid>`;
  GRANT MODIFY, SELECT ON TABLE <catalog.schema.table> TO `<service-principal-uuid>`;
  ```

### High Lambda Errors

- Increase Lambda memory and timeout if processing large batches
- Check Zerobus endpoint connectivity
- Verify table schema matches the protobuf definition

