provider "aws" {
  region = var.aws_region
  default_tags {
    tags = {
      "DeployedBy"  = "Terraform"
      "Service"     = "zerobus-generic-ingestor"
      "Environment" = terraform.workspace
      "Version"     = "0.1.0"
    }
  }
}

