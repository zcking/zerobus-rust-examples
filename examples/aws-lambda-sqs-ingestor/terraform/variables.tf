locals {
  lambda_zip_path = "${path.module}/../../../target/lambda/aws-lambda-sqs-ingestor/bootstrap.zip"
}

variable "aws_region" {
  description = "AWS region for resources"
  type        = string
  default     = "us-west-2"
}

variable "function_name" {
  description = "Name of the Lambda function"
  type        = string
  default     = "zerobus-sqs-ingestor"
}

variable "queue_name" {
  description = "Name of the SQS queue"
  type        = string
  default     = "zerobus-ingestion-queue"
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
  description = "Unity Catalog table name (e.g., zach_king.zerobus.sqs_messages)"
  type        = string
  default     = "zach_king.zerobus.sqs_messages"
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

variable "batch_size" {
  description = "Maximum number of records to retrieve from SQS in a single batch"
  type        = number
  default     = 10
}

variable "maximum_batching_window_in_seconds" {
  description = "Maximum batching window in seconds for SQS event source mapping"
  type        = number
  default     = 0
}

variable "visibility_timeout_seconds" {
  description = "SQS queue visibility timeout in seconds (should be >= Lambda timeout)"
  type        = number
  default     = 300
}

variable "dlq_max_receive_count" {
  description = "Maximum number of times a message can be received before being sent to DLQ"
  type        = number
  default     = 3
}

variable "log_retention_days" {
  description = "CloudWatch log retention in days"
  type        = number
  default     = 7
}
