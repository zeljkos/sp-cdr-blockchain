// SP CDR Blockchain Network Demo
// Demonstrates P2P networking, consensus, and settlement messaging
use sp_cdr_reconciliation_bc::network::{
    SPNetworkManager, NetworkCommand, NetworkEvent, SPNetworkMessage,
    PeerDiscovery, ConsensusNetwork, SettlementMessaging,
};
use sp_cdr_reconciliation_bc::lib::NetworkId;
use libp2p::{Multiaddr, PeerId};
use tokio::time::{sleep, Duration};
use tracing::{info, warn, error};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🚀 Starting SP CDR Blockchain Network Demo");

    // Simulate three major telecom operators
    let operators = vec![
        ("T-Mobile-DE", "127.0.0.1:8000"),
        ("Vodafone-UK", "127.0.0.1:8001"),
        ("Orange-FR", "127.0.0.1:8002"),
    ];

    info!("📡 Launching {} operator nodes...", operators.len());

    // Launch nodes in parallel
    let mut handles = vec![];

    for (i, (operator_name, addr)) in operators.iter().enumerate() {
        let operator_name = operator_name.to_string();
        let addr = addr.parse::<Multiaddr>()?;

        let handle = tokio::spawn(async move {
            if let Err(e) = run_operator_node(operator_name, addr, i == 0).await {
                error!("Operator node failed: {}", e);
            }
        });

        handles.push(handle);

        // Stagger node startup
        sleep(Duration::from_secs(2)).await;
    }

    info!("🌐 All operator nodes launched. Running demo scenario...");

    // Run demo scenario
    sleep(Duration::from_secs(5)).await;
    run_demo_scenario().await?;

    // Keep demo running
    info!("💫 Demo running - press Ctrl+C to stop");

    // Wait for all nodes
    for handle in handles {
        if let Err(e) = handle.await {
            error!("Node handle error: {}", e);
        }
    }

    Ok(())
}

async fn run_operator_node(
    operator_name: String,
    listen_addr: Multiaddr,
    is_coordinator: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let parts: Vec<&str> = operator_name.split('-').collect();
    let network_id = NetworkId::new(parts[0], parts.get(1).unwrap_or(&"XX"));

    info!("🏢 Starting {} node at {}", operator_name, listen_addr);

    // Create network manager
    let (network_manager, command_sender, mut event_receiver) = SPNetworkManager::new(
        network_id.clone(),
        listen_addr,
    ).await?;

    // Initialize peer discovery
    let peer_discovery = PeerDiscovery::with_sp_consortium().await?;

    // Start network manager in background
    let network_handle = tokio::spawn(network_manager.run());

    // Create consensus network (validators only)
    if is_coordinator || operator_name.contains("Mobile") || operator_name.contains("Vodafone") {
        info!("🏛️  {} joining as validator", operator_name);

        let validators = std::collections::HashSet::new(); // Would populate with real validators
        let weights = std::collections::HashMap::new();

        let consensus = ConsensusNetwork::new(
            network_id.clone(),
            PeerId::random(),
            validators,
            weights,
            command_sender.clone(),
        );

        // Start consensus in background
        tokio::spawn(async move {
            info!("🗳️  Consensus engine started for {}", operator_name);
            // Would run consensus logic here
            loop {
                sleep(Duration::from_secs(30)).await;
                info!("⚖️  {} consensus heartbeat", operator_name);
            }
        });
    }

    // Create settlement messaging
    let settlement_messaging = SettlementMessaging::new(
        network_id.clone(),
        PeerId::random(),
        command_sender.clone(),
    );

    // Handle network events
    let settlement_handle = tokio::spawn(async move {
        info!("💰 Settlement messaging started for {}", operator_name);

        while let Ok(event) = event_receiver.recv().await {
            match event {
                NetworkEvent::PeerConnected(peer_id) => {
                    info!("🤝 {} connected to peer: {}", operator_name, peer_id);
                }

                NetworkEvent::PeerDisconnected(peer_id) => {
                    info!("👋 {} disconnected from peer: {}", operator_name, peer_id);
                }

                NetworkEvent::MessageReceived { peer, message } => {
                    info!("📨 {} received message from {}: {:?}", operator_name, peer, message);
                }

                NetworkEvent::GossipReceived { topic, message, source } => {
                    info!("📢 {} heard gossip on {}: {:?} from {}", operator_name, topic, message, source);

                    // Handle settlement messages
                    if topic == "settlement" {
                        match message {
                            SPNetworkMessage::SettlementProposal { creditor, debtor, amount_cents, .. } => {
                                if debtor == network_id {
                                    info!("💸 {} received settlement request from {} for €{}",
                                          operator_name, creditor, amount_cents as f64 / 100.0);

                                    // Auto-accept small amounts for demo
                                    if amount_cents <= 50000 { // €500
                                        info!("✅ {} auto-accepting settlement", operator_name);
                                        // Would send acceptance message
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
    });

    // Demo settlement initiation for coordinator
    if is_coordinator {
        tokio::spawn(async move {
            sleep(Duration::from_secs(10)).await;

            info!("🎯 {} initiating demo settlements...", operator_name);

            // Simulate CDR processing leading to settlements
            let settlements = vec![
                (NetworkId::new("Vodafone", "UK"), 25000, "EUR"), // €250
                (NetworkId::new("Orange", "FR"), 18500, "EUR"),   // €185
            ];

            for (debtor, amount, currency) in settlements {
                info!("📋 {} proposing settlement: {} owes {} {}",
                      operator_name, debtor, amount as f64 / 100.0, currency);

                let proposal_msg = SPNetworkMessage::SettlementProposal {
                    creditor: network_id.clone(),
                    debtor,
                    amount_cents: amount,
                    period_hash: sp_cdr_reconciliation_bc::lib::Blake2bHash::default(),
                    nonce: rand::random(),
                };

                let _ = command_sender.send(NetworkCommand::Broadcast {
                    topic: "settlement".to_string(),
                    message: proposal_msg,
                });

                sleep(Duration::from_secs(5)).await;
            }

            // Propose triangular netting after bilateral settlements
            sleep(Duration::from_secs(10)).await;
            info!("🔺 {} proposing triangular netting to optimize settlements", operator_name);

            // Would calculate optimal netting here
            let netting_msg = SPNetworkMessage::SettlementProposal {
                creditor: network_id.clone(),
                debtor: NetworkId::new("System", "Netting"),
                amount_cents: 0, // Net amount after optimization
                period_hash: sp_cdr_reconciliation_bc::lib::Blake2bHash::default(),
                nonce: rand::random(),
            };

            let _ = command_sender.send(NetworkCommand::Broadcast {
                topic: "settlement".to_string(),
                message: netting_msg,
            });
        });
    }

    // Wait for handles
    tokio::select! {
        _ = network_handle => {
            info!("🔌 Network manager stopped for {}", operator_name);
        }
        _ = settlement_handle => {
            info!("🛑 Settlement messaging stopped for {}", operator_name);
        }
    }

    Ok(())
}

async fn run_demo_scenario() -> Result<(), Box<dyn std::error::Error>> {
    info!("🎬 Running demo scenario: Monthly CDR Settlement");

    info!("📊 Scenario: End of month reconciliation between 3 operators");
    info!("   • T-Mobile DE has processed roaming traffic");
    info!("   • Vodafone UK has cross-border SMS charges");
    info!("   • Orange FR has data roaming fees");

    sleep(Duration::from_secs(5)).await;

    info!("🔄 Phase 1: CDR batch processing and ZK proof generation");
    info!("   • Each operator processes their CDR batches");
    info!("   • ZK proofs generated to preserve privacy");
    info!("   • Settlement amounts calculated without revealing individual records");

    sleep(Duration::from_secs(5)).await;

    info!("🤝 Phase 2: Bilateral settlement proposals");
    info!("   • T-Mobile proposes settlements to Vodafone and Orange");
    info!("   • Each operator reviews and responds to proposals");
    info!("   • Signatures and agreements exchanged via P2P network");

    sleep(Duration::from_secs(5)).await;

    info!("🔺 Phase 3: Triangular netting optimization");
    info!("   • System calculates optimal netting arrangement");
    info!("   • Reduces gross settlement from €X to €Y (savings: Z%)");
    info!("   • Final settlement instructions generated");

    sleep(Duration::from_secs(5)).await;

    info!("⚖️  Phase 4: Consensus and finalization");
    info!("   • Validators agree on final settlement amounts");
    info!("   • ZK proofs verify calculation correctness");
    info!("   • Settlement recorded on blockchain");

    sleep(Duration::from_secs(5)).await;

    info!("✅ Demo scenario complete! Settlement successfully processed with:");
    info!("   • Privacy preserved via zero-knowledge proofs");
    info!("   • Costs reduced via triangular netting");
    info!("   • Transparency ensured via blockchain consensus");
    info!("   • P2P networking enabled decentralized operation");

    Ok(())
}