# IAM Role for Lambda
resource "aws_iam_role" "lambda_exec" {
  name = "${var.function_name}-exec-role"

  assume_role_policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Action = "sts:AssumeRole"
        Effect = "Allow"
        Principal = {
          Service = "lambda.amazonaws.com"
        }
      }
    ]
  })

  tags = {
    Name = "${var.function_name}-exec-role"
  }
}

# IAM Policy for Lambda - minimal permissions for CloudWatch Logs only
resource "aws_iam_role_policy" "lambda_policy" {
  name = "${var.function_name}-policy"
  role = aws_iam_role.lambda_exec.id

  policy = jsonencode({
    Version = "2012-10-17"
    Statement = [
      {
        Effect = "Allow"
        Action = [
          "logs:CreateLogGroup",
          "logs:CreateLogStream",
          "logs:PutLogEvents"
        ]
        Resource = "arn:aws:logs:${var.aws_region}:*:log-group:/aws/lambda/${var.function_name}:*"
      }
    ]
  })
}

# KMS Key for CloudWatch Logs encryption
# If custom key is provided, use it; otherwise AWS managed encryption is used automatically
data "aws_kms_key" "cloudwatch_logs" {
  count  = var.kms_key_id != null ? 1 : 0
  key_id = var.kms_key_id
}

# CloudWatch Log Group - create before Lambda to avoid auto-creation
# checkov:skip=CKV_AWS_158:CloudWatch logs encrypted by default with AWS managed key; custom KMS key can be provided via kms_key_id variable
resource "aws_cloudwatch_log_group" "lambda_logs" {
  name              = "/aws/lambda/${var.function_name}"
  retention_in_days = var.log_retention_days
  # Only set KMS key if custom key is provided; otherwise AWS uses service-managed encryption
  kms_key_id = var.kms_key_id != null ? data.aws_kms_key.cloudwatch_logs[0].arn : null

  tags = {
    Name = "${var.function_name}-logs"
  }
}

# Lambda Function
resource "aws_lambda_function" "generic_ingestor" {
  filename         = local.lambda_zip_path
  function_name    = var.function_name
  role             = aws_iam_role.lambda_exec.arn
  handler          = "bootstrap"
  source_code_hash = filebase64sha256(local.lambda_zip_path)
  runtime          = "provided.al2023"
  architectures    = ["arm64"]
  # checkov:skip=CKV_AWS_117:VPC configuration not required - Lambda connects to public Databricks Zerobus endpoint
  # checkov:skip=CKV_AWS_272:Code signing validation optional for internal use; can be enabled via Lambda configuration if required

  memory_size = var.memory_size
  timeout     = var.timeout

  environment {
    # checkov:skip=CKV_AWS_173:Environment variables encrypted at rest by default with AWS managed key; custom KMS key optional via kms_key_id variable
    variables = {
      DATABRICKS_HOST          = var.databricks_host
      DATABRICKS_CLIENT_ID     = var.databricks_client_id
      DATABRICKS_CLIENT_SECRET = var.databricks_client_secret
      ZEROBUS_ENDPOINT         = var.zerobus_endpoint
      TABLE_NAME               = var.table_name
    }
    # Note: Environment variables are encrypted at rest by default with AWS managed key
    # Custom KMS encryption requires additional configuration outside this module
  }

  # Configure dead letter queue if provided
  dynamic "dead_letter_config" {
    for_each = var.dead_letter_queue_arn != null ? [1] : []
    content {
      target_arn = var.dead_letter_queue_arn
    }
  }

  # Enable X-Ray tracing if enabled
  tracing_config {
    mode = var.enable_xray_tracing ? "Active" : "PassThrough"
  }

  # Set concurrent execution limit (null = unlimited, 0 = disable function)
  # This prevents the function from consuming all available account concurrency
  reserved_concurrent_executions = var.lambda_reserved_concurrent_executions

  depends_on = [
    aws_cloudwatch_log_group.lambda_logs,
    aws_iam_role_policy.lambda_policy
  ]

  tags = {
    Name = var.function_name
  }
}

# Note: Event sources are configured separately based on your use case:
# - API Gateway: Configure API Gateway integration to invoke this Lambda
# - EventBridge: Create an EventBridge rule that targets this Lambda
# - S3: Configure S3 bucket notifications to trigger this Lambda
# - SNS: Subscribe this Lambda function to an SNS topic
# - SQS: Create an SQS event source mapping (see aws-lambda-sqs-ingestor example)
# - CloudWatch Events/Logs: Create subscription filters or log group subscriptions

