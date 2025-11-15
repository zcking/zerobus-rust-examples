# Hello World Example

A basic example demonstrating the fundamental workflow of using the Databricks Zerobus SDK for Rust.

For comprehensive documentation, see the official [Databricks Zerobus Ingest documentation](https://docs.databricks.com/aws/en/ingestion/lakeflow-connect/zerobus-ingest?language=Rust%C2%A0SDK).

## What This Example Does

This example walks through the complete lifecycle of a Zerobus ingestion session:

1. **SDK Initialization**: Creates a `ZerobusSdk` instance with your workspace endpoints
2. **Stream Creation**: Opens an authenticated connection to a Unity Catalog table
3. **Message Encoding**: Encodes a simple message using Protocol Buffers
4. **Record Ingestion**: Sends the encoded message to Zerobus
5. **Acknowledgment**: Waits for confirmation that the message was received
6. **Graceful Shutdown**: Flushes pending records and closes the stream

## Prerequisites

- Rust 1.70 or later
- [buf](https://buf.build) CLI tool: `brew install bufbuild/buf/buf`
- `zerobus-generate` tool (see [root README](../../README.md) for installation)
- Databricks workspace with Zerobus enabled, service principal credentials, and Unity Catalog table

## Quick Start

See the [root README](../../README.md) for initial workspace setup (service principal creation, environment variables, etc.).

### 1. Create Your Unity Catalog Table

```sql
CREATE TABLE main.default.zerobus_hello_world (
    msg STRING,
    timestamp BIGINT
) USING DELTA;
```

Grant permissions to your service principal (see root README for details).

### 2. Generate and Compile Protocol Buffers

```bash
cd examples/hello-world

# Generate .proto file from Unity Catalog table
make proto-generate

# Compile .proto to Rust bindings and descriptors
make proto-compile

# Or run both steps together:
make proto
```

This creates:
- `proto/zerobus_hello_world.proto` - Source schema (committed to git)
- `gen/rust/zerobus_hello_world.rs` - Rust message structs (generated)
- `gen/descriptors/zerobus_hello_world.descriptor` - Runtime descriptor (generated)

### 3. Build and Run

```bash
make build
make run
```

Expected output:
```
Zerobus Hello World Example
=============================

Initializing Zerobus SDK...
Creating stream to table: main.default.zerobus_hello_world
Stream created successfully!

Sending message: Hello, Zerobus!
Message sent, waiting for acknowledgment...
Message acknowledged successfully!
Stream flushed.

Stream closed. Hello World example complete!
```