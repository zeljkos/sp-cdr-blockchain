# Multi-Party Trusted Setup Ceremony

This Docker Compose setup emulates a **real multi-party trusted setup ceremony** between telecom operators using containerized participants.

## 🎯 What This Demonstrates

### Production Security Properties
- **Multi-party computation**: No single party can compromise the ceremony
- **Sequential contributions**: Each participant adds entropy to previous contributions
- **Independent verification**: External auditor verifies ceremony integrity
- **Persistent storage**: All ceremony data survives container restarts
- **Transparency**: Complete audit trail of all contributions

### Simulated Telecom Operators
- 🇩🇪 **T-Mobile Deutschland** - German mobile operator
- 🇬🇧 **Vodafone UK** - British mobile operator
- 🇫🇷 **Orange France** - French mobile operator
- 🔍 **Independent Verifier** - External auditor

## 🚀 Quick Start

```bash
cd docker
./start-trusted-setup.sh
```

## 📊 Ceremony Flow

```
Time   Container              Action
------ --------------------- ----------------------------------------
T+0s   setup-coordinator     Initialize ceremony parameters
T+15s  participant-tmobile   T-Mobile contributes entropy
T+30s  participant-vodafone  Vodafone contributes entropy
T+45s  participant-orange    Orange contributes entropy
T+90s  ceremony-verifier     Independent audit of ceremony
T+150s validator-1,2,3       Blockchain starts using ceremony keys
```

## 🔍 Monitoring

### Ceremony Progress
- **Coordinator Dashboard**: http://localhost:9000
- **T-Mobile Participant**: http://localhost:9010
- **Vodafone Participant**: http://localhost:9020
- **Orange Participant**: http://localhost:9030
- **Independent Verifier**: http://localhost:9100

### Blockchain (Post-Ceremony)
- **Validator 1**: http://localhost:8081
- **Validator 2**: http://localhost:8091
- **Validator 3**: http://localhost:8101

## 📂 Persistent Storage Structure

```
persistent_data/
├── ceremony_coordinator/     # Coordinator state
├── participant_tmobile/      # T-Mobile private data
├── participant_vodafone/     # Vodafone private data
├── participant_orange/       # Orange private data
├── ceremony_verifier/        # Verifier audit data
├── shared_ceremony/          # Shared ceremony state
├── shared_zkp_keys/          # Final ZK proving/verifying keys
├── validator-1/              # Blockchain validator data
├── validator-2/              # Blockchain validator data
└── validator-3/              # Blockchain validator data
```

## 🔐 Security Properties

### What Makes This Secure
1. **No single point of failure**: All 3 operators must participate
2. **Sequential verification**: Each contribution is verified before next
3. **Entropy accumulation**: Each participant adds unpredictable randomness
4. **Independent audit**: External verifier confirms ceremony integrity
5. **Persistent proof**: Complete transcript stored for future verification

### What This Simulates
- **Real Powers of Tau**: Each participant contributes to parameters
- **Multi-party signatures**: All contributions are cryptographically signed
- **Ceremony transcript**: Immutable record of all contributions
- **External verification**: Independent party confirms no corruption

## 🧪 Production Readiness

### For Demo/Testing ✅
- Demonstrates multi-party security model
- Shows sequential contribution process
- Provides independent verification
- Creates real ceremony transcript

### For Production 🔧 (Needs Enhancement)
- [ ] Real network communication between operators
- [ ] Hardware security modules (HSMs) for key protection
- [ ] Legal agreements between telecom operators
- [ ] Regulatory compliance documentation
- [ ] Real-time monitoring and alerting
- [ ] Backup and disaster recovery procedures

## 🛠️ Commands

```bash
# Start ceremony
./start-trusted-setup.sh

# Monitor ceremony progress
docker compose -f docker-compose.trusted-setup-persistent.yml logs -f

# Check specific participant
docker compose -f docker-compose.trusted-setup-persistent.yml logs participant-tmobile

# Stop ceremony
docker compose -f docker-compose.trusted-setup-persistent.yml down

# Clean ceremony data
rm -rf ../persistent_data/shared_ceremony/*
rm -rf ../persistent_data/shared_zkp_keys/*
```

## 🎯 Success Criteria

The ceremony succeeds when:
- ✅ All 3 telecom operators contribute entropy
- ✅ Independent verifier confirms ceremony integrity
- ✅ Final ZK keys are generated and saved
- ✅ Blockchain validators start using ceremony keys
- ✅ Complete audit trail is preserved

## 🔍 Verification

After ceremony completion, you can verify:
- **Ceremony transcript**: `persistent_data/shared_ceremony/ceremony_transcript.json`
- **ZK proving keys**: `persistent_data/shared_zkp_keys/settlement_proving_key.bin`
- **ZK verifying keys**: `persistent_data/shared_zkp_keys/settlement_verifying_key.bin`
- **Audit report**: `persistent_data/ceremony_verifier/audit_report.json`

This demonstrates production-grade multi-party security in a containerized environment.