# Default target
.PHONY: help
help:
	@echo "AWS Lambda Generic Ingestor - Available commands:"
	@echo ""
	@echo "Build & Package:"
	@echo "  make build           - Build Lambda function"
	@echo "  make package         - Package Lambda function into zip file"
	@echo "  make clean           - Clean build artifacts"
	@echo ""
	@echo "Protocol Buffers:"
	@echo "  make proto           - Generate proto files from Unity Catalog table"
	@echo "                        (requires DATABRICKS_HOST, DATABRICKS_CLIENT_ID,"
	@echo "                         DATABRICKS_CLIENT_SECRET, TABLE_NAME)"
	@echo ""
	@echo "Terraform:"
	@echo "  make terraform-init  - Initialize Terraform"
	@echo "  make terraform-plan  - Plan Terraform changes"
	@echo "  make terraform-apply - Apply Terraform configuration"
	@echo "  make terraform-destroy - Destroy Terraform resources"
	@echo ""
	@echo "Testing:"
	@echo "  make serve           - Serve Lambda function locally"
	@echo "  make invoke          - Invoke Lambda function locally with test event"
	@echo "  make test-logs       - Tail CloudWatch logs"
	@echo "  make test-query      - Query Unity Catalog table (requires Databricks CLI)"
	@echo ""
	@echo "Utilities:"
	@echo "  make deps-check      - Check if required dependencies are installed"

# Variables
LAMBDA_PACKAGE_NAME = aws-generic-ingestor
LAMBDA_BUILD_DIR = ../../target/lambda/$(LAMBDA_PACKAGE_NAME)
LAMBDA_ZIP_FILE = $(LAMBDA_BUILD_DIR)/bootstrap.zip
TERRAFORM_DIR = terraform
PROTO_DIR = proto

# Build Lambda function
.PHONY: build
build: ARGS = --arm64
build:
	@echo "Building Lambda function..."
	@echo "Add ARGS='--arm64 --release' to compile a release build"
	@if ! command -v cargo-lambda &> /dev/null; then \
		echo "Error: cargo-lambda is not installed."; \
		echo "Install it with: brew install cargo-lambda/tap/cargo-lambda"; \
		exit 1; \
	fi
	cargo lambda build --output-format zip $(ARGS)
	ls -hl $(LAMBDA_BUILD_DIR)
	@echo "Build complete!"

# Clean build artifacts
.PHONY: clean
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	@echo "Clean complete!"

# Generate Protocol Buffer files from Unity Catalog table
.PHONY: proto
proto:
	@echo "Generating Protocol Buffer files..."
	@if ! command -v zerobus-generate &> /dev/null; then \
		echo "Error: zerobus-generate is not installed."; \
		echo "Install it by:"; \
		echo "  1. Clone: git clone https://github.com/databricks/zerobus-sdk-rs.git"; \
		echo "  2. Build: cd zerobus-sdk-rs/tools/generate_files && cargo build --release"; \
		echo "  3. Install: cp target/release/generate_files ~/.cargo/bin/zerobus-generate"; \
		exit 1; \
	fi
	@if [ -z "$$DATABRICKS_HOST" ] || [ -z "$$DATABRICKS_CLIENT_ID" ] || [ -z "$$DATABRICKS_CLIENT_SECRET" ] || [ -z "$$TABLE_NAME" ]; then \
		echo "Error: Required environment variables not set:"; \
		echo "  DATABRICKS_HOST"; \
		echo "  DATABRICKS_CLIENT_ID"; \
		echo "  DATABRICKS_CLIENT_SECRET"; \
		echo "  TABLE_NAME"; \
		exit 1; \
	fi
	zerobus-generate \
		--uc-endpoint $$DATABRICKS_HOST \
		--client-id $$DATABRICKS_CLIENT_ID \
		--client-secret $$DATABRICKS_CLIENT_SECRET \
		--table $$TABLE_NAME \
		--output-dir $(PROTO_DIR)
	@echo "Proto files generated in $(PROTO_DIR)/"

# Terraform commands
.PHONY: terraform-init
terraform-init:
	@echo "Initializing Terraform..."
	cd $(TERRAFORM_DIR) && terraform init

.PHONY: terraform-plan
terraform-plan: terraform-init
	@echo "Planning Terraform changes..."
	cd $(TERRAFORM_DIR) && terraform plan

.PHONY: terraform-apply
terraform-apply: terraform-init build
	@echo "Applying Terraform configuration..."
	@if [ ! -f $(LAMBDA_ZIP_FILE) ]; then \
		echo "Error: Lambda zip file not found at $(LAMBDA_ZIP_FILE)"; \
		echo "Run 'make build' first"; \
		exit 1; \
	fi
	cd $(TERRAFORM_DIR) && terraform apply

.PHONY: terraform-destroy
terraform-destroy:
	@echo "Destroying Terraform resources..."
	cd $(TERRAFORM_DIR) && terraform destroy

# Testing commands
.PHONY: serve
serve:
	@echo "Serving Lambda function locally..."
	cargo lambda watch

.PHONY: invoke
invoke: ARGS = --data-example api-gateway-request
invoke:
	@echo "Invoking Lambda function locally..."
	cargo lambda invoke aws-generic-ingestor $(ARGS)

.PHONY: test-logs
test-logs:
	@echo "Tailing CloudWatch logs..."
	@if ! command -v terraform &> /dev/null; then \
		echo "Error: terraform not found in PATH"; \
		exit 1; \
	fi
	@if ! command -v aws &> /dev/null; then \
		echo "Error: AWS CLI not found in PATH"; \
		exit 1; \
	fi
	@LOG_GROUP=$$(cd $(TERRAFORM_DIR) && terraform output -raw cloudwatch_log_group_name 2>/dev/null); \
	if [ -z "$$LOG_GROUP" ]; then \
		echo "Error: Could not get log group name. Make sure Terraform resources are deployed."; \
		exit 1; \
	fi; \
	aws logs tail "$$LOG_GROUP" --follow

.PHONY: test-query
test-query:
	@echo "Querying Unity Catalog table..."
	@if ! command -v databricks &> /dev/null; then \
		echo "Error: Databricks CLI not found in PATH"; \
		echo "Install it from: https://docs.databricks.com/dev-tools/cli/index.html"; \
		exit 1; \
	fi
	@if [ -z "$$TABLE_NAME" ]; then \
		echo "Error: TABLE_NAME environment variable not set"; \
		exit 1; \
	fi
	@echo "SELECT * FROM $$TABLE_NAME ORDER BY ingested_at DESC LIMIT 10;" | databricks sql execute

# Check if required dependencies are installed
.PHONY: deps-check
deps-check:
	@echo "Checking dependencies..."
	@MISSING=0; \
	if ! command -v cargo &> /dev/null; then \
		echo "✗ cargo not found"; \
		MISSING=1; \
	else \
		echo "✓ cargo found"; \
	fi; \
	if ! command -v cargo-lambda &> /dev/null; then \
		echo "✗ cargo-lambda not found (install with: brew install cargo-lambda/tap/cargo-lambda)"; \
		MISSING=1; \
	else \
		echo "✓ cargo-lambda found"; \
	fi; \
	if ! command -v zerobus-generate &> /dev/null; then \
		echo "✗ zerobus-generate not found (see README.md for installation)"; \
		MISSING=1; \
	else \
		echo "✓ zerobus-generate found"; \
	fi; \
	if ! command -v terraform &> /dev/null; then \
		echo "✗ terraform not found"; \
		MISSING=1; \
	else \
		echo "✓ terraform found"; \
	fi; \
	if ! command -v aws &> /dev/null; then \
		echo "✗ aws CLI not found"; \
		MISSING=1; \
	else \
		echo "✓ aws CLI found"; \
	fi; \
	if [ $$MISSING -eq 1 ]; then \
		echo ""; \
		echo "Some dependencies are missing. Please install them before proceeding."; \
		exit 1; \
	else \
		echo ""; \
		echo "All required dependencies are installed!"; \
	fi

