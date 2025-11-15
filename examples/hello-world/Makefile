# Default target
.PHONY: help
help:
	@echo "Hello World Example - Available commands:"
	@echo ""
	@echo "Build:"
	@echo "  make build           - Build the example (generates proto files first)"
	@echo "  make run             - Run the example"
	@echo "  make clean           - Clean build artifacts and generated code"
	@echo ""
	@echo "Protocol Buffers:"
	@echo "  make proto           - Generate proto files and compile to Rust bindings"
	@echo "  make proto-generate  - Generate .proto from Unity Catalog table"
	@echo "                        (requires DATABRICKS_HOST, DATABRICKS_CLIENT_ID,"
	@echo "                         DATABRICKS_CLIENT_SECRET, TABLE_NAME)"
	@echo "  make proto-compile   - Compile .proto files to Rust bindings with buf"
	@echo ""
	@echo "Utilities:"
	@echo "  make deps-check      - Check if required dependencies are installed"

# Variables
PROTO_DIR := proto
GEN_DIR := gen

# Full proto workflow: generate .proto from UC, then compile with buf
.PHONY: proto
proto: proto-generate proto-compile

# Step 1: Generate .proto from Unity Catalog table using zerobus-generate
.PHONY: proto-generate
proto-generate:
	@echo "Generating .proto files from Unity Catalog..."
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
	@echo "Cleaning up old generated .rs and .descriptor files..."
	@rm -f $(PROTO_DIR)/*.rs $(PROTO_DIR)/*.descriptor
	@echo "Proto files generated in $(PROTO_DIR)/"
	@echo "Note: Old .rs and .descriptor files removed. Run 'make proto-compile' to regenerate with buf."

# Step 2: Compile .proto to Rust bindings and descriptor files using buf
.PHONY: proto-compile
proto-compile:
	@echo "Compiling proto files with buf..."
	@if ! command -v buf &> /dev/null; then \
		echo "Error: buf is not installed."; \
		echo "Install it with:"; \
		echo "  macOS: brew install bufbuild/buf/buf"; \
		echo "  Linux: https://buf.build/docs/installation"; \
		exit 1; \
	fi
	@echo "Generating Rust bindings..."
	buf generate $(PROTO_DIR)/
	@echo "Generating descriptor files..."
	@mkdir -p $(GEN_DIR)/descriptors
	@for proto_file in $(PROTO_DIR)/*.proto; do \
		if [ -f "$$proto_file" ]; then \
			base_name=$$(basename "$$proto_file" .proto); \
			buf build "$$proto_file" -o "$(GEN_DIR)/descriptors/$${base_name}.descriptor" --as-file-descriptor-set; \
		fi; \
	done
	@echo "Generated code in $(GEN_DIR)/"
	@echo "  - Rust bindings: $(GEN_DIR)/rust/"
	@echo "  - Descriptors: $(GEN_DIR)/descriptors/"

# Build the example (auto-generate proto if needed)
.PHONY: build
build: proto-compile
	@echo "Building hello-world example..."
	cargo build

# Run the example
.PHONY: run
run: proto-compile
	@echo "Running hello-world example..."
	cargo run

# Clean build artifacts and generated code
.PHONY: clean
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	@echo "Cleaning generated code..."
	rm -rf $(GEN_DIR)
	@echo "Clean complete!"

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
	if ! command -v buf &> /dev/null; then \
		echo "✗ buf not found (install with: brew install bufbuild/buf/buf)"; \
		MISSING=1; \
	else \
		echo "✓ buf found"; \
	fi; \
	if ! command -v zerobus-generate &> /dev/null; then \
		echo "✗ zerobus-generate not found (see README.md for installation)"; \
		MISSING=1; \
	else \
		echo "✓ zerobus-generate found"; \
	fi; \
	if [ $$MISSING -eq 1 ]; then \
		echo ""; \
		echo "Some dependencies are missing. Please install them before proceeding."; \
		exit 1; \
	else \
		echo ""; \
		echo "All required dependencies are installed!"; \
	fi
