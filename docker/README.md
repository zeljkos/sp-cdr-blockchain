# SP CDR Blockchain Docker Demo

This directory contains everything needed to run a 3-validator SP CDR blockchain demo using Docker containers.

## Quick Start

```bash
cd docker

# Make scripts executable
chmod +x *.sh

# Start the blockchain network
./start.sh

# Test the network (in another terminal)
./test.sh

# Monitor the network (in another terminal)
./monitor.sh

# Stop the network
./stop.sh
```

## Files Overview

- **`Dockerfile`** - Container definition for SP CDR blockchain nodes
- **`docker-compose.yml`** - 3-validator network configuration
- **`start.sh`** - Start the blockchain network
- **`stop.sh`** - Stop the blockchain network
- **`test.sh`** - Run comprehensive tests
- **`monitor.sh`** - Real-time monitoring dashboard
- **`README.md`** - This file

## Network Architecture

### Validators
- **Validator 1** (Bootstrap): `localhost:8080` (P2P), `localhost:8081` (API)
- **Validator 2**: `localhost:8090` (P2P), `localhost:8091` (API)
- **Validator 3**: `localhost:8100` (P2P), `localhost:8101` (API)

### Internal Network
- Network: `172.30.0.0/16`
- Validator 1: `172.30.0.10`
- Validator 2: `172.30.0.11`
- Validator 3: `172.30.0.12`

## Manual Commands

### Start Network
```bash
docker compose up --build
```

### Start in Background
```bash
docker compose up -d --build
```

### View Logs
```bash
# All validators
docker compose logs -f

# Specific validator
docker compose logs -f validator-1
```

### Execute Commands in Containers
```bash
# Test cryptography
docker exec sp-validator-1 ./target/release/test-real-crypto

# Run CDR pipeline demo
docker exec sp-validator-1 ./target/release/cdr-pipeline-demo

# Enter container shell
docker exec -it sp-validator-1 bash
```

### Check Container Status
```bash
docker compose ps
docker stats
```

### Clean Up
```bash
# Stop and remove containers
docker compose down

# Remove containers and volumes
docker compose down -v

# Remove everything including images
docker compose down -v --rmi all
```

## Testing Scenarios

### 1. Basic Connectivity Test
```bash
./test.sh
```

### 2. Manual API Tests
```bash
# Health checks
curl http://localhost:8081/health
curl http://localhost:8091/health
curl http://localhost:8101/health

# Network peers (if endpoint exists)
curl http://localhost:8081/peers
curl http://localhost:8091/peers
curl http://localhost:8101/peers
```

### 3. Cryptographic Validation
```bash
docker exec sp-validator-1 ./target/release/test-real-crypto
docker exec sp-validator-2 ./target/release/test-real-crypto
docker exec sp-validator-3 ./target/release/test-real-crypto
```

### 4. CDR Pipeline Demo
```bash
docker exec sp-validator-1 ./target/release/cdr-pipeline-demo
```

## Data Persistence

Blockchain data is stored in:
- `./data/validator-1/` - Validator 1 data
- `./data/validator-2/` - Validator 2 data
- `./data/validator-3/` - Validator 3 data

Data persists between container restarts.

## Troubleshooting

### Containers Won't Start
```bash
# Check Docker daemon
docker info

# Check for port conflicts
netstat -tuln | grep -E "8080|8081|8090|8091|8100|8101"

# View detailed logs
docker compose logs
```

### Network Issues
```bash
# Check container networking
docker network ls
docker network inspect docker_sp_blockchain_net

# Test internal connectivity
docker exec sp-validator-2 ping validator-1
docker exec sp-validator-3 ping validator-1
```

### Performance Issues
```bash
# Monitor resources
docker stats

# Check host resources
free -h
top
```

### Clean Restart
```bash
./stop.sh
docker system prune -f
./start.sh
```

## Expected Output

When running successfully, you should see:
- ✅ All 3 validators running and healthy
- ✅ Cryptographic tests passing (3.5M+ keys/second)
- ✅ Network connectivity between all validators
- ✅ CDR pipeline executing successfully
- ✅ Consensus being established between validators

## Production Notes

This demo configuration is optimized for development and testing. For production deployment:

1. Use proper secrets management for validator keys
2. Configure persistent volumes for data
3. Set up proper monitoring and alerting
4. Use production-ready logging
5. Configure firewall rules
6. Use TLS certificates for API endpoints

## Support

For issues or questions:
1. Check the logs: `docker compose logs`
2. Run the test suite: `./test.sh`
3. Review the main deployment guide: `../deployment/README.md`