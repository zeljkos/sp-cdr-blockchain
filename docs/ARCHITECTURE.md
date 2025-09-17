# SP CDR Reconciliation Blockchain - Architecture

```
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                           SP CDR RECONCILIATION BLOCKCHAIN ARCHITECTURE                        │
└─────────────────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                                    TELECOM OPERATORS                                           │
├─────────────────────┬─────────────────────┬─────────────────────┬─────────────────────────────┤
│   T-Mobile Germany  │   Vodafone UK       │   Orange France     │   Other SP Operators       │
│                     │                     │                     │                             │
│  ┌─────────────┐   │  ┌─────────────┐   │  ┌─────────────┐   │  ┌─────────────┐             │
│  │ CDR Systems │   │  │ CDR Systems │   │  │ CDR Systems │   │  │ CDR Systems │             │
│  └─────┬───────┘   │  └─────┬───────┘   │  └─────┬───────┘   │  └─────┬───────┘             │
│        │           │        │           │        │           │        │                     │
│  ┌─────▼───────┐   │  ┌─────▼───────┐   │  ┌─────▼───────┐   │  ┌─────▼───────┐             │
│  │ ZK Provers  │   │  │ ZK Provers  │   │  │ ZK Provers  │   │  │ ZK Provers  │             │
│  │ (Privacy)   │   │  │ (Privacy)   │   │  │ (Privacy)   │   │  │ (Privacy)   │             │
│  └─────────────┘   │  └─────────────┘   │  └─────────────┘   │  └─────────────┘             │
└─────────┬───────────┴─────────┬───────────┴─────────┬───────────┴─────────┬─────────────────┘
          │                     │                     │                     │
          │ Encrypted CDR       │ Encrypted CDR       │ Encrypted CDR       │ Encrypted CDR
          │ + ZK Proofs         │ + ZK Proofs         │ + ZK Proofs         │ + ZK Proofs
          ▼                     ▼                     ▼                     ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                                 SP CDR BLOCKCHAIN NODE                                        │
├─────────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                               │
│  ┌─────────────────────────────────────────────────────────────────────────────────────┐    │
│  │                              API LAYER                                               │    │
│  ├─────────────────┬─────────────────┬─────────────────┬─────────────────────────────┤    │
│  │  CDR Submission │ Batch Status    │ Settlement API  │ Blockchain Explorer         │    │
│  │  POST /cdr      │ GET /batch/{id} │ POST /settle    │ GET /blocks, /txns          │    │
│  └─────────────────┴─────────────────┴─────────────────┴─────────────────────────────┘    │
│                                        │                                                    │
│  ┌─────────────────────────────────────▼─────────────────────────────────────────────┐    │
│  │                        BLOCKCHAIN INTEGRATION LAYER                               │    │
│  │                                                                                   │    │
│  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────┐                  │    │
│  │  │ CDR Processing  │  │ Settlement Calc │  │ Multi-Sig Mgmt  │                  │    │
│  │  │ - Batch CDRs    │  │ - Netting Algo  │  │ - BLS Aggregate │                  │    │
│  │  │ - Triangulation │  │ - Exchange Rate │  │ - Threshold Sig │                  │    │
│  │  └─────────────────┘  └─────────────────┘  └─────────────────┘                  │    │
│  └─────────────────────────────────▼─────────────────────────────────────────────────┘    │
│                                    │                                                        │
│  ┌─────────────────────────────────▼─────────────────────────────────────────────────┐    │
│  │                           SMART CONTRACT LAYER                                   │    │
│  │                                                                                   │    │
│  │  ┌─────────────────────────────────────────────────────────────────────────────┐ │    │
│  │  │                      CONTRACT EXECUTION ENGINE                              │ │    │
│  │  │                                                                             │ │    │
│  │  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐                   │ │    │
│  │  │  │ Contract VM │  │ Bytecode    │  │ Gas Metering    │                   │ │    │
│  │  │  │ (Stack)     │  │ Execution   │  │ & Limits        │                   │ │    │
│  │  │  └─────────────┘  └─────────────┘  └─────────────────┘                   │ │    │
│  │  └─────────────────────────────────────────────────────────────────────────────┘ │    │
│  │                                    │                                             │    │
│  │  ┌─────────────────────────────────▼─────────────────────────────────────────┐   │    │
│  │  │                    SETTLEMENT CONTRACTS                                   │   │    │
│  │  │                                                                           │   │    │
│  │  │  ┌─────────────────┐  ┌─────────────────┐  ┌─────────────────────────┐   │   │    │
│  │  │  │ CDR Privacy     │  │ Settlement      │  │ Multi-Party Validation │   │   │    │
│  │  │  │ Contract        │  │ Calculation     │  │ Contract                │   │   │    │
│  │  │  │ - Verify ZK     │  │ Contract        │  │ - BLS Verification     │   │   │    │
│  │  │  │ - Batch Commit  │  │ - Netting Logic │  │ - Threshold Consensus  │   │   │    │
│  │  │  └─────────────────┘  └─────────────────┘  └─────────────────────────┘   │   │    │
│  │  └─────────────────────────────────────────────────────────────────────────────┘   │    │
│  └─────────────────────────────────▼─────────────────────────────────────────────────────┘    │
│                                    │                                                          │
│  ┌─────────────────────────────────▼─────────────────────────────────────────────────────┐  │
│  │                           CRYPTOGRAPHIC LAYER                                         │  │
│  │                                                                                        │  │
│  │  ┌─────────────────────────────┐  ┌─────────────────────────────┐                     │  │
│  │  │     ZK PROOF SYSTEM         │  │     BLS SIGNATURE SYSTEM    │                     │  │
│  │  │                             │  │                             │                     │  │
│  │  │  ┌─────────────────────┐   │  │  ┌─────────────────────────┐ │                     │  │
│  │  │  │ arkworks/BN254      │   │  │  │ BLS12-381 Curve         │ │                     │  │
│  │  │  │ - Groth16 Proofs    │   │  │  │ - Individual Signatures │ │                     │  │
│  │  │  │ - Settlement VK     │   │  │  │ - Aggregate Signatures  │ │                     │  │
│  │  │  │ - CDR Privacy VK    │   │  │  │ - Threshold Schemes     │ │                     │  │
│  │  │  └─────────────────────┘   │  │  └─────────────────────────┘ │                     │  │
│  │  │                             │  │                             │                     │  │
│  │  │  ┌─────────────────────┐   │  │  ┌─────────────────────────┐ │                     │  │
│  │  │  │ Poseidon Hash       │   │  │  │ Multi-Party Validation  │ │                     │  │
│  │  │  │ - MNT4/MNT6 Fields  │   │  │  │ - Network Operators     │ │                     │  │
│  │  │  │ - Circuit-Friendly  │   │  │  │ - Consortium Consensus  │ │                     │  │
│  │  │  └─────────────────────┘   │  │  └─────────────────────────┘ │                     │  │
│  │  └─────────────────────────────┘  └─────────────────────────────┘                     │  │
│  └─────────────────────────────────▼─────────────────────────────────────────────────────────┘  │
│                                    │                                                            │
│  ┌─────────────────────────────────▼─────────────────────────────────────────────────────────┐│
│  │                         BLOCKCHAIN CONSENSUS LAYER                                        ││
│  │                                                                                            ││
│  │  ┌─────────────────────────────────────────────────────────────────────────────────────┐ ││
│  │  │                           ALBATROSS CONSENSUS                                        │ ││
│  │  │                                                                                     │ ││
│  │  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────────┐  ┌─────────────────────┐   │ ││
│  │  │  │ Micro Blocks│  │ Macro Blocks│  │ Validator Set   │  │ Epoch Management    │   │ ││
│  │  │  │ - CDR Txns  │  │ - Finality  │  │ - SP Operators  │  │ - Election Blocks   │   │ ││
│  │  │  │ - Fast Conf │  │ - Elections │  │ - Voting Power  │  │ - 32 Block Epochs   │   │ ││
│  │  │  └─────────────┘  └─────────────┘  └─────────────────┘  └─────────────────────┘   │ ││
│  │  └─────────────────────────────────────────────────────────────────────────────────────┘ ││
│  │                                       │                                                   ││
│  │  ┌─────────────────────────────────────▼─────────────────────────────────────────────┐   ││
│  │  │                          TRANSACTION PROCESSING                                   │   ││
│  │  │                                                                                   │   ││
│  │  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────┐  │   ││
│  │  │  │ CDR Records │  │ Settlement  │  │ Validator   │  │ Contract Deployment     │  │   ││
│  │  │  │ Transaction │  │ Transaction │  │ Transaction │  │ Transaction             │  │   ││
│  │  │  └─────────────┘  └─────────────┘  └─────────────┘  └─────────────────────────┘  │   ││
│  │  └─────────────────────────────────────────────────────────────────────────────────────┘   ││
│  └─────────────────────────────────────▼─────────────────────────────────────────────────────────┘│
│                                        │                                                          │
│  ┌─────────────────────────────────────▼─────────────────────────────────────────────────────────┐│
│  │                              STORAGE LAYER                                                    ││
│  │                                                                                                ││
│  │  ┌─────────────────────────────────────────────────────────────────────────────────────────┐ ││
│  │  │                             MDBX DATABASE                                                │ ││
│  │  │                                                                                         │ ││
│  │  │  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  ┌─────────────────────────┐  │ ││
│  │  │  │ Blockchain    │  │ Contract      │  │ ZK Keys       │  │ Operator Certificates  │  │ ││
│  │  │  │ Data          │  │ State         │  │ Storage       │  │ & BLS Keys             │  │ ││
│  │  │  │ - Blocks      │  │ - VM State    │  │ - Verifying   │  │ - Network IDs          │  │ ││
│  │  │  │ - Transactions│  │ - Contract    │  │   Keys        │  │ - Public Keys          │  │ ││
│  │  │  │ - State Roots │  │   Code        │  │ - Proving     │  │ - Voting Power         │  │ ││
│  │  │  │ - Head Ptrs   │  │ - Storage     │  │   Keys        │  │                        │  │ ││
│  │  │  └───────────────┘  └───────────────┘  └───────────────┘  └─────────────────────────┘  │ ││
│  │  └─────────────────────────────────────────────────────────────────────────────────────────┘ ││
│  │                                                                                                ││
│  │  ┌─────────────────────────────────────────────────────────────────────────────────────────┐ ││
│  │  │                          STORAGE INTERFACES                                              │ ││
│  │  │                                                                                         │ ││
│  │  │  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  ┌─────────────────────────┐  │ ││
│  │  │  │ ChainStore    │  │ ContractStore │  │ KeyStore      │  │ ConfigurationStore      │  │ ││
│  │  │  │ Trait         │  │ Trait         │  │ Trait         │  │ Trait                   │  │ ││
│  │  │  └───────────────┘  └───────────────┘  └───────────────┘  └─────────────────────────┘  │ ││
│  │  └─────────────────────────────────────────────────────────────────────────────────────────┘ ││
│  └─────────────────────────────────────────────────────────────────────────────────────────────┘│
└─────────────────────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                      DATA FLOWS                                                    │
└─────────────────────────────────────────────────────────────────────────────────────────────────┘

1. CDR SUBMISSION FLOW:
   Operator → ZK Prover → Encrypted CDR + Proof → API → CDR Processing → Smart Contracts → Blockchain

2. SETTLEMENT FLOW:
   Multiple CDRs → Triangular Netting → Settlement Calculation → Multi-Sig Validation → Settlement TX

3. CONSENSUS FLOW:
   Transaction Pool → Micro Block → Validator Signatures → Block Finalization → MDBX Storage

4. PRIVACY FLOW:
   Raw CDR → ZK Circuit → Privacy Proof → Verification → Settlement Without Data Exposure

┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                 NETWORK ARCHITECTURE                                               │
└─────────────────────────────────────────────────────────────────────────────────────────────────┘

                    ┌─────────────────────────────────────────────────────────────────┐
                    │                     SP CONSORTIUM NETWORK                       │
                    └─────────────────────────────────────────────────────────────────┘
                                                    │
        ┌─────────────────────┬─────────────────────┼─────────────────────┬─────────────────────┐
        │                     │                     │                     │                     │
   ┌────▼────┐          ┌────▼────┐          ┌────▼────┐          ┌────▼────┐          ┌────▼────┐
   │ Node 1  │◄────────►│ Node 2  │◄────────►│ Node 3  │◄────────►│ Node 4  │◄────────►│ Node N  │
   │T-Mobile │          │Vodafone │          │ Orange  │          │ Other   │          │ Future  │
   │Germany  │          │   UK    │          │ France  │          │   SP    │          │   SP    │
   └─────────┘          └─────────┘          └─────────┘          └─────────┘          └─────────┘
        │                     │                     │                     │                     │
        │                     │                     │                     │                     │
   ┌────▼────┐          ┌────▼────┐          ┌────▼────┐          ┌────▼────┐          ┌────▼────┐
   │ Local   │          │ Local   │          │ Local   │          │ Local   │          │ Local   │
   │ MDBX    │          │ MDBX    │          │ MDBX    │          │ MDBX    │          │ MDBX    │
   │Database │          │Database │          │Database │          │Database │          │Database │
   └─────────┘          └─────────┘          └─────────┘          └─────────┘          └─────────┘

┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                   KEY TECHNOLOGIES                                                 │
├─────────────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                                   │
│ • Programming Language: Rust (async/await, tokio runtime)                                        │
│ • Consensus Algorithm: Albatross (Micro/Macro blocks, PBFT-style finality)                       │
│ • Storage Engine: MDBX (Memory-mapped database, ACID transactions)                               │
│ • Zero-Knowledge: arkworks (BN254 curve, Groth16 proofs)                                         │
│ • Digital Signatures: BLS12-381 (Aggregatable signatures, threshold schemes)                     │
│ • Hash Functions: Blake2b (General), Poseidon (ZK-friendly)                                      │
│ • Serialization: serde (JSON/Binary), bincode (Efficient binary)                                 │
│ • Network Protocol: Custom P2P over TCP/TLS                                                      │
│ • Smart Contracts: Custom VM (Stack-based, gas metered)                                          │
│                                                                                                   │
└─────────────────────────────────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────────────────────────────────┐
│                               SECURITY & PRIVACY FEATURES                                          │
├─────────────────────────────────────────────────────────────────────────────────────────────────┤
│                                                                                                   │
│ 1. DATA PRIVACY:                                                                                  │
│    • CDR data encrypted before blockchain submission                                              │
│    • Zero-knowledge proofs prove settlement correctness without revealing CDR details            │
│    • Operators only see their own settlement amounts, not other parties' data                    │
│                                                                                                   │
│ 2. CRYPTOGRAPHIC SECURITY:                                                                        │
│    • BLS signature aggregation for efficient multi-party validation                              │
│    • Threshold signatures prevent single points of failure                                        │
│    • Groth16 proofs provide succinctness and efficient verification                              │
│                                                                                                   │
│ 3. CONSENSUS SECURITY:                                                                            │
│    • Albatross provides instant finality through macro blocks                                     │
│    • Byzantine fault tolerance up to 1/3 malicious validators                                     │
│    • Epoch-based validator set updates with election blocks                                       │
│                                                                                                   │
│ 4. NETWORK SECURITY:                                                                              │
│    • TLS encryption for all P2P communications                                                    │
│    • Certificate-based operator authentication                                                    │
│    • Isolated consortium network (no public access)                                              │
│                                                                                                   │
└─────────────────────────────────────────────────────────────────────────────────────────────────┘