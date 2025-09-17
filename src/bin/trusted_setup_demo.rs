// Trusted Setup Ceremony Demo
// Demonstrates real key generation for SP consortium
use sp_cdr_reconciliation_bc::zkp::trusted_setup::TrustedSetupCeremony;
use ark_std::rand::thread_rng;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    tracing::info!("🔐 SP Consortium Trusted Setup Ceremony Demo");

    // Create ceremony with test keys directory
    let keys_dir = std::path::PathBuf::from("./test_ceremony_keys");
    let mut ceremony = TrustedSetupCeremony::sp_consortium_ceremony(keys_dir.clone());

    tracing::info!("🏗️  Running trusted setup ceremony...");
    let mut rng = thread_rng();

    // Run the full ceremony
    let transcript = ceremony.run_ceremony(&mut rng).await?;

    tracing::info!("✅ Ceremony completed!");
    tracing::info!("📋 Ceremony ID: {}", transcript.ceremony_id);
    tracing::info!("👥 Participants: {:?}", transcript.participants);
    tracing::info!("🔍 Verification Status: {:?}", transcript.verification_status);

    // Verify the ceremony
    let verification_result = ceremony.verify_ceremony().await?;
    tracing::info!("🔍 Ceremony verification: {}", verification_result);

    // Get ceremony statistics
    let stats = ceremony.get_ceremony_stats().await?;
    tracing::info!("📊 Ceremony Statistics:");
    for (circuit_id, circuit_stats) in &stats.circuits {
        if let Some((pk_size, vk_size)) = circuit_stats.key_sizes {
            tracing::info!("   • {}: PK: {} bytes, VK: {} bytes",
                          circuit_id, pk_size, vk_size);
        }
    }

    // Test key loading
    if ceremony.keys_exist("cdr_privacy").await {
        let (_pk, _vk) = ceremony.load_circuit_keys("cdr_privacy").await?;
        tracing::info!("🔑 Successfully loaded CDR Privacy circuit keys");
    }

    if ceremony.keys_exist("settlement_calculation").await {
        let (_pk, _vk) = ceremony.load_circuit_keys("settlement_calculation").await?;
        tracing::info!("🔑 Successfully loaded Settlement Calculation circuit keys");
    }

    tracing::info!("🎉 Trusted setup ceremony demo complete!");
    tracing::info!("💾 Keys saved to: {:?}", keys_dir);

    Ok(())
}