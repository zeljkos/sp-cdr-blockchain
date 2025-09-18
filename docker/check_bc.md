 Available inspection targets:

  # 📦 View blockchain blocks
  docker exec -it sp-validator-1 ./target/release/sp-cdr-node inspect --target blocks

  # 💳 View transactions
  docker exec -it sp-validator-1 ./target/release/sp-cdr-node inspect --target transactions

  # 📞 View CDR data and ZK setup
  docker exec -it sp-validator-1 ./target/release/sp-cdr-node inspect --target cdrs

  # 💰 View settlement processing
  docker exec -it sp-validator-1 ./target/release/sp-cdr-node inspect --target settlements

  # 📈 View system statistics
  docker exec -it sp-validator-1 ./target/release/sp-cdr-node inspect --target stats

  # 🔍 Inspect specific block
  docker exec -it sp-validator-1 ./target/release/sp-cdr-node inspect --target blocks --id 0

  # 📁 Use different data directory
  docker exec -it sp-validator-2 ./target/release/sp-cdr-node inspect --data-dir /app/data --target stats

