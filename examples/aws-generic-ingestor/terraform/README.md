# Terraform Configuration for AWS Generic Ingestor

This Terraform configuration deploys the AWS Lambda function for generic event ingestion into Databricks Zerobus.

## Prerequisites

- Terraform >= 1.0
- AWS CLI configured with appropriate credentials
- Lambda function built and packaged (run `make build` from the example directory)

## Configuration

### Variables

Create a `terraform.tfvars` file with the following variables:

```hcl
aws_region = "us-west-2"
function_name = "zerobus-generic-ingestor"

databricks_host = "https://myworkspace.cloud.databricks.com"
databricks_client_id = "your-client-id"
databricks_client_secret = "your-client-secret"
zerobus_endpoint = "https://<workspace_id>.zerobus.<region>.cloud.databricks.com"

table_name = "zach_king.zerobus.aws_raw_events"

# Optional: adjust Lambda configuration
memory_size = 512
timeout = 60
log_retention_days = 7
```

### Required Variables

- `databricks_host` - Databricks workspace URL
- `databricks_client_id` - Service principal client ID
- `databricks_client_secret` - Service principal secret
- `zerobus_endpoint` - Zerobus gRPC endpoint

### Optional Variables

- `aws_region` - AWS region (default: "us-west-2")
- `function_name` - Lambda function name (default: "zerobus-generic-ingestor")
- `table_name` - Unity Catalog table name (default: "zach_king.zerobus.aws_raw_events")
- `memory_size` - Lambda memory in MB (default: 512)
- `timeout` - Lambda timeout in seconds (default: 60)
- `log_retention_days` - CloudWatch log retention (default: 7)

## Deployment

### Initialize Terraform

```bash
terraform init
```

### Plan Changes

```bash
terraform plan
```

### Apply Configuration

```bash
terraform apply
```

### Destroy Resources

```bash
terraform destroy
```

## Resources Created

- **Lambda Function**: The main function that processes events
- **IAM Role**: Execution role for the Lambda function with CloudWatch Logs permissions
- **CloudWatch Log Group**: Stores Lambda function logs

## Connecting Event Sources

After deploying the Lambda function, you can connect it to any AWS event source. Here are examples for common sources:

### API Gateway

1. Create an API Gateway REST or HTTP API
2. Create a method (GET, POST, etc.) and configure it to invoke the Lambda function
3. Use the Lambda function ARN from the outputs

### EventBridge

Create an EventBridge rule that targets the Lambda function:

```hcl
resource "aws_cloudwatch_event_rule" "example" {
  name        = "trigger-generic-ingestor"
  description = "Trigger Lambda function from EventBridge"

  event_pattern = jsonencode({
    source      = ["aws.s3"]
    detail-type = ["Object Created"]
  })
}

resource "aws_cloudwatch_event_target" "lambda" {
  rule      = aws_cloudwatch_event_rule.example.name
  target_id = "GenericIngestorTarget"
  arn       = aws_lambda_function.generic_ingestor.arn
}

resource "aws_lambda_permission" "allow_eventbridge" {
  statement_id  = "AllowExecutionFromEventBridge"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.generic_ingestor.function_name
  principal     = "events.amazonaws.com"
  source_arn    = aws_cloudwatch_event_rule.example.arn
}
```

### S3

Configure S3 bucket notifications to trigger the Lambda:

```hcl
resource "aws_s3_bucket_notification" "lambda_trigger" {
  bucket = "your-bucket-name"

  lambda_function {
    lambda_function_arn = aws_lambda_function.generic_ingestor.arn
    events              = ["s3:ObjectCreated:*"]
  }
}

resource "aws_lambda_permission" "allow_s3" {
  statement_id  = "AllowExecutionFromS3Bucket"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.generic_ingestor.function_name
  principal     = "s3.amazonaws.com"
  source_arn    = "arn:aws:s3:::your-bucket-name"
}
```

### SNS

Subscribe the Lambda function to an SNS topic:

```hcl
resource "aws_sns_topic_subscription" "lambda" {
  topic_arn = "arn:aws:sns:region:account:topic-name"
  protocol  = "lambda"
  endpoint  = aws_lambda_function.generic_ingestor.arn
}

resource "aws_lambda_permission" "allow_sns" {
  statement_id  = "AllowExecutionFromSNS"
  action        = "lambda:InvokeFunction"
  function_name = aws_lambda_function.generic_ingestor.function_name
  principal     = "sns.amazonaws.com"
  source_arn    = "arn:aws:sns:region:account:topic-name"
}
```

### SQS

For SQS, consider using the `aws-lambda-sqs-ingestor` example which includes partial batch response handling. However, you can still use this generic ingestor with SQS:

```hcl
resource "aws_lambda_event_source_mapping" "sqs_trigger" {
  event_source_arn = "arn:aws:sqs:region:account:queue-name"
  function_name    = aws_lambda_function.generic_ingestor.function_name
  batch_size       = 10
}
```

## Outputs

After deployment, you can retrieve outputs:

```bash
# Get Lambda function ARN
terraform output -raw lambda_function_arn

# Get Lambda function name
terraform output -raw lambda_function_name

# Get CloudWatch log group name
terraform output -raw cloudwatch_log_group_name
```

## Security Considerations

- All sensitive variables (credentials, secrets) are marked as `sensitive = true`
- IAM role follows least privilege principle (only CloudWatch Logs permissions)
- Lambda function uses ARM64 architecture for cost efficiency
- CloudWatch Log Group has configurable retention period

## Troubleshooting

### Lambda Function Not Working

1. Check CloudWatch logs:
   ```bash
   LOG_GROUP=$(terraform output -raw cloudwatch_log_group_name)
   aws logs tail "$LOG_GROUP" --follow
   ```

2. Verify environment variables are set correctly
3. Ensure the Lambda zip file exists at the expected path
4. Check IAM role permissions

### Checkov Compliance

This configuration is designed to pass Checkov security checks:

```bash
checkov --directory terraform --compact
```

The configuration includes:
- Proper IAM role with least privilege
- CloudWatch Log Group with retention
- Resource tags for organization
- Secure defaults

