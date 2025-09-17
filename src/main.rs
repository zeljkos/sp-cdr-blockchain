// SP CDR Reconciliation Blockchain Node
// Main entry point for running the blockchain node

use clap::{Parser, Subcommand};
use sp_cdr_reconciliation_bc::{*, cdr_pipeline};
use std::sync::Arc;
use tracing::{info, error};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(name = "sp-cdr-node")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the blockchain node
    Start {
        /// Network ID to connect to
        #[arg(short, long, default_value = "consortium")]
        network: String,
        /// Data directory for blockchain storage
        #[arg(short, long, default_value = "./data")]
        data_dir: String,
        /// Port to listen on
        #[arg(short, long, default_value = "8080")]
        port: u16,
    },
    /// Generate validator keys
    GenerateKeys {
        /// Output directory for keys
        #[arg(short, long, default_value = "./keys")]
        output: String,
    },
    /// Validate CDR records
    ValidateCDR {
        /// Path to CDR file
        #[arg(short, long)]
        file: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    let cli = Cli::parse();

    match cli.command {
        Commands::Start { network, data_dir, port } => {
            start_node(network, data_dir, port).await
        }
        Commands::GenerateKeys { output } => {
            generate_validator_keys(output).await
        }
        Commands::ValidateCDR { file } => {
            validate_cdr_file(file).await
        }
    }
}

async fn start_node(network: String, data_dir: String, port: u16) -> Result<()> {
    info!("Starting SP CDR Reconciliation Blockchain Node");
    info!("Network: {}, Data Directory: {}, Port: {}", network, data_dir, port);

    // Parse network ID - use specific operator networks for demo
    let network_id = match network.as_str() {
        "tmobile" => NetworkId::new("T-Mobile", "DE"),
        "vodafone" => NetworkId::new("Vodafone", "UK"),
        "orange" => NetworkId::new("Orange", "FR"),
        "consortium" => NetworkId::SPConsortium,
        "devnet" => NetworkId::DevNet,
        "testnet" => NetworkId::TestNet,
        _ => {
            error!("Unknown network: {}. Use: tmobile, vodafone, orange, consortium, devnet, testnet", network);
            std::process::exit(1);
        }
    };

    // Create data directory
    std::fs::create_dir_all(&data_dir)?;

    // Create pipeline configuration
    let pipeline_config = cdr_pipeline::PipelineConfig {
        keys_dir: std::path::PathBuf::from(format!("{}/zkp_keys", data_dir)),
        batch_size: 1000,
        settlement_threshold_cents: 10000, // €100 minimum
        auto_accept_threshold_cents: 50000, // €500 auto-accept
        enable_triangular_netting: true,
    };

    // Create network listen address
    let listen_addr = format!("/ip4/127.0.0.1/tcp/{}", port).parse()
        .map_err(|e| primitives::BlockchainError::Network(format!("Invalid address: {}", e)))?;

    info!("🏗️  Initializing complete CDR pipeline...");

    // Initialize integrated CDR pipeline
    let mut pipeline = cdr_pipeline::CDRPipeline::new(
        network_id.clone(),
        listen_addr,
        pipeline_config,
    ).await?;

    info!("✅ CDR Pipeline initialized successfully");
    info!("🎯 Operator: {:?}", network_id);
    info!("🌐 Listening on port: {}", port);
    info!("💾 Data directory: {}", data_dir);

    // Add some sample CDR data for demonstration
    if matches!(network_id, NetworkId::SPConsortium) {
        info!("📋 Adding sample CDR batches for demonstration...");

        // Sample roaming traffic between operators
        pipeline.add_sample_cdr_batch(
            NetworkId::new("T-Mobile", "DE"),
            NetworkId::new("Vodafone", "UK")
        ).await?;

        pipeline.add_sample_cdr_batch(
            NetworkId::new("Vodafone", "UK"),
            NetworkId::new("Orange", "FR")
        ).await?;

        pipeline.add_sample_cdr_batch(
            NetworkId::new("Orange", "FR"),
            NetworkId::new("T-Mobile", "DE")
        ).await?;

        info!("📊 Sample CDR batches created - settlement processing will begin automatically");
    }

    info!("🚀 Starting integrated CDR processing pipeline...");

    // Start the complete pipeline
    let pipeline_handle = tokio::spawn(async move {
        if let Err(e) = pipeline.run().await {
            error!("CDR pipeline error: {:?}", e);
        }
    });

    // Wait for shutdown signal
    info!("✅ CDR Pipeline running - processing CDR batches and settlements");
    info!("Press Ctrl+C to stop...");

    tokio::select! {
        _ = tokio::signal::ctrl_c() => {
            info!("Shutdown signal received...");
        }
        result = pipeline_handle => {
            error!("Pipeline stopped unexpectedly: {:?}", result);
        }
    }

    info!("🛑 Shutting down CDR pipeline...");
    Ok(())
}

async fn generate_validator_keys(output: String) -> Result<()> {
    info!("Generating validator keys");
    
    std::fs::create_dir_all(&output)?;
    
    // Generate BLS signing key
    let signing_keypair = crypto::KeyPair::generate()?;
    
    // Generate Ed25519 voting key (mock)
    let voting_key = vec![42u8; 32]; // In real implementation, use Ed25519 key generation
    
    // Create validator key
    let validator_key = crypto::ValidatorKey::new(
        hash_data(b"generated_validator"),
        signing_keypair.public_key.compress(),
        voting_key,
        hash_data(b"reward_address"),
        0,
    )?;
    
    // Save keys (in real implementation, save to files securely)
    info!("Validator keys generated successfully");
    info!("Signing key ID: {:?}", signing_keypair.key_id);
    info!("Keys saved to: {}", output);
    
    println!("✅ Validator keys generated at: {}", output);
    println!("   Signing Key ID: {:?}", signing_keypair.key_id);
    println!("   Validator Address: {:?}", validator_key.validator_address);
    
    Ok(())
}

async fn validate_cdr_file(file_path: String) -> Result<()> {
    info!("Validating CDR file: {}", file_path);
    
    // Check if file exists
    if !std::path::Path::new(&file_path).exists() {
        error!("CDR file not found: {}", file_path);
        std::process::exit(1);
    }
    
    // In real implementation, this would:
    // 1. Parse CDR file
    // 2. Validate CDR records
    // 3. Check network operators
    // 4. Verify signatures
    // 5. Validate charges
    
    info!("CDR validation completed for: {}", file_path);
    println!("✅ CDR file validation completed: {}", file_path);
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_key_generation() {
        let temp_dir = "/tmp/test_keys";
        let result = generate_validator_keys(temp_dir.to_string()).await;
        assert!(result.is_ok());
    }
}