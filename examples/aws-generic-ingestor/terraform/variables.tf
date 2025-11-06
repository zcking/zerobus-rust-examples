locals {
  lambda_zip_path = "${path.module}/../../../target/lambda/aws-generic-ingestor/bootstrap.zip"
}

variable "aws_region" {
  description = "AWS region for resources"
  type        = string
  default     = "us-west-2"
}

variable "function_name" {
  description = "Name of the Lambda function"
  type        = string
  default     = "zerobus-generic-ingestor"
}

variable "databricks_host" {
  description = "Databricks workspace URL (e.g., https://myworkspace.cloud.databricks.com)"
  type        = string
  sensitive   = true
}

variable "databricks_client_id" {
  description = "Databricks service principal client ID"
  type        = string
  sensitive   = true
}

variable "databricks_client_secret" {
  description = "Databricks service principal client secret"
  type        = string
  sensitive   = true
}

variable "zerobus_endpoint" {
  description = "Zerobus gRPC endpoint (e.g., https://<workspace_id>.zerobus.<region>.cloud.databricks.com)"
  type        = string
  sensitive   = true
}

variable "table_name" {
  description = "Unity Catalog table name (e.g., zach_king.zerobus.aws_raw_events)"
  type        = string
  default     = "zach_king.zerobus.aws_raw_events"
}

variable "memory_size" {
  description = "Lambda function memory size in MB"
  type        = number
  default     = 512
}

variable "timeout" {
  description = "Lambda function timeout in seconds"
  type        = number
  default     = 60
}

variable "log_retention_days" {
  description = "CloudWatch log retention in days (minimum 1 for cost savings, recommended 365 for compliance)"
  type        = number
  default     = 365
}

variable "kms_key_id" {
  description = "KMS key ID for encrypting CloudWatch logs and Lambda environment variables. If not provided, AWS managed key will be used"
  type        = string
  default     = null
}

variable "lambda_reserved_concurrent_executions" {
  description = "Reserved concurrent executions for the Lambda function. Set to limit concurrent executions (null = unlimited, but Checkov recommends setting a limit)"
  type        = number
  default     = 100
}

variable "enable_xray_tracing" {
  description = "Enable X-Ray tracing for the Lambda function"
  type        = bool
  default     = false
}

variable "dead_letter_queue_arn" {
  description = "ARN of the SQS queue or SNS topic to use as a Dead Letter Queue. If not provided, DLQ will not be configured"
  type        = string
  default     = null
}

