provider "aws" {
  region = var.aws_region
  default_tags {
    tags = {
      "DeployedBy"  = "Terraform"
      "Service"     = "zerobus-sqs-ingestor"
      "Environment" = terraform.workspace
      "Version"     = "0.1.0"
    }
  }
}