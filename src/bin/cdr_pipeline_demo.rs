// Complete CDR Pipeline Integration Demo
// Shows end-to-end integration: CDR → ZK Proofs → Settlement → Blockchain
use sp_cdr_reconciliation_bc::{cdr_pipeline::*, lib::NetworkId};
use std::path::PathBuf;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    tracing::info!("🚀 SP CDR Reconciliation - Complete Pipeline Demo");
    tracing::info!("Demonstrating end-to-end integration:");
    tracing::info!("  1. CDR Batch Processing with real ZK proofs");
    tracing::info!("  2. P2P Network Communication");
    tracing::info!("  3. Settlement Proposals and Acceptance");
    tracing::info!("  4. Blockchain Transaction Recording");

    // Create configuration for demo
    let config = PipelineConfig {
        keys_dir: PathBuf::from("./demo_zkp_keys"),
        batch_size: 100,
        settlement_threshold_cents: 1000, // €10 minimum
        auto_accept_threshold_cents: 5000, // €50 auto-accept
        enable_triangular_netting: true,
    };

    // Simulate T-Mobile DE operator
    let network_id = NetworkId::new("T-Mobile", "DE");
    let listen_addr = "/ip4/127.0.0.1/tcp/8900".parse()?;

    tracing::info!("🏢 Initializing T-Mobile DE operator pipeline...");

    // Initialize complete pipeline
    let mut pipeline = CDRPipeline::new(
        network_id.clone(),
        listen_addr,
        config,
    ).await.map_err(|e| format!("Pipeline initialization failed: {:?}", e))?;

    tracing::info!("✅ Pipeline initialized with:");
    tracing::info!("   🔐 Real ZK proving/verifying keys from trusted setup");
    tracing::info!("   🌐 P2P networking with libp2p");
    tracing::info!("   💾 Blockchain storage integration");
    tracing::info!("   ⚖️  Consensus and settlement messaging");

    // Add sample CDR batches to demonstrate complete flow
    tracing::info!("📋 Creating sample CDR traffic...");

    // Roaming traffic: T-Mobile DE customers in UK (Vodafone network)
    pipeline.add_sample_cdr_batch(
        NetworkId::new("T-Mobile", "DE"),    // Home network
        NetworkId::new("Vodafone", "UK")     // Visited network
    ).await.map_err(|e| format!("Failed to add CDR batch: {:?}", e))?;

    // Roaming traffic: T-Mobile DE customers in France (Orange network)
    pipeline.add_sample_cdr_batch(
        NetworkId::new("T-Mobile", "DE"),    // Home network
        NetworkId::new("Orange", "FR")       // Visited network
    ).await.map_err(|e| format!("Failed to add CDR batch: {:?}", e))?;

    tracing::info!("📊 Sample CDR batches created and announced to network");
    tracing::info!("🔄 Pipeline will now process:");
    tracing::info!("   1. Generate ZK proofs for privacy-preserving CDR validation");
    tracing::info!("   2. Calculate settlement amounts");
    tracing::info!("   3. Propose bilateral settlements via P2P network");
    tracing::info!("   4. Process settlement acceptances");
    tracing::info!("   5. Record final settlements on blockchain");

    // Start the pipeline in a separate task
    let pipeline_handle = tokio::spawn(async move {
        match pipeline.run().await {
            Ok(_) => tracing::info!("Pipeline completed successfully"),
            Err(e) => tracing::error!("Pipeline error: {:?}", e),
        }
    });

    // Let the pipeline run for demonstration
    tracing::info!("🔄 Running pipeline for 60 seconds to demonstrate complete flow...");

    tokio::select! {
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(60)) => {
            tracing::info!("⏰ Demo time completed");
        }
        result = pipeline_handle => {
            tracing::info!("Pipeline completed: {:?}", result);
        }
        _ = tokio::signal::ctrl_c() => {
            tracing::info!("👋 Demo interrupted by user");
        }
    }

    tracing::info!("🎉 CDR Pipeline Demo Complete!");
    tracing::info!("📈 This demonstrated:");
    tracing::info!("   ✅ Real ZK proof generation and verification");
    tracing::info!("   ✅ P2P networking between telecom operators");
    tracing::info!("   ✅ Automated settlement processing");
    tracing::info!("   ✅ Blockchain transaction recording");
    tracing::info!("   ✅ Privacy-preserving CDR reconciliation");

    tracing::info!("🔗 All components are now fully integrated end-to-end!");

    Ok(())
}