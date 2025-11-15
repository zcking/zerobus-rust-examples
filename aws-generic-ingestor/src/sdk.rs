use anyhow::Result;
use databricks_zerobus_ingest_sdk::ZerobusSdk;
use std::sync::OnceLock;

// Global SDK instance for reuse across Lambda invocations
static SDK: OnceLock<ZerobusSdk> = OnceLock::new();

/// Initialize the Zerobus SDK (called once per Lambda container)
pub fn init_sdk() -> Result<&'static ZerobusSdk> {
    SDK.get_or_init(|| {
        let zerobus_endpoint = std::env::var("ZEROBUS_ENDPOINT")
            .expect("ZEROBUS_ENDPOINT environment variable must be set");
        let databricks_host = std::env::var("DATABRICKS_HOST")
            .expect("DATABRICKS_HOST environment variable must be set");

        ZerobusSdk::new(zerobus_endpoint, databricks_host)
            .expect("Failed to initialize ZerobusSdk")
    });
    Ok(SDK.get().expect("SDK should be initialized"))
}

