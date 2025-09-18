// BCE API Server
// Standalone server for ingesting BCE records from operator billing systems

use sp_cdr_reconciliation_bc::{
    bce_pipeline::*,
    api::bce_ingestion::*,
    primitives::primitives::NetworkId,
};
use std::{sync::Arc, path::PathBuf};
use tokio::sync::Mutex;
use tracing::{info, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🚀 SP BCE Record Ingestion API Server");
    info!("Ready to receive BCE records from operator billing systems");

    // Configuration
    let api_port = 9090;
    let network_port = 9000;
    let keys_dir = PathBuf::from("./api_zkp_keys");

    // Create BCE pipeline configuration
    let config = PipelineConfig {
        keys_dir,
        batch_size: 100,
        settlement_threshold_cents: 10000, // €100 minimum
        auto_accept_threshold_cents: 50000, // €500 auto-accept
        enable_triangular_netting: true,
        is_bootstrap: true,
    };

    // Initialize BCE pipeline (simplified for API server)
    let listen_addr: libp2p::Multiaddr = format!("/ip4/127.0.0.1/tcp/{}", network_port).parse().unwrap();

    info!("🏗️  Initializing BCE Pipeline for API server...");
    let mut pipeline = BCEPipeline::new(
        NetworkId::SPConsortium,
        listen_addr,
        config,
    ).await?;

    info!("✅ BCE Pipeline initialized");

    // Wrap pipeline in Arc<Mutex> for API sharing
    let pipeline = Arc::new(Mutex::new(pipeline));

    // Create and start BCE ingestion API
    let api_server = BCEIngestAPI::new(pipeline.clone(), api_port);

    // Print curl examples for testing
    print_curl_examples(api_port);

    info!("🌐 Starting BCE API server on port {}...", api_port);
    info!("📡 Ready to receive BCE records from operator billing systems");

    // Start the API server (this will run indefinitely)
    if let Err(e) = api_server.start().await {
        error!("❌ Failed to start BCE API server: {:?}", e);
        return Err(e);
    }

    Ok(())
}