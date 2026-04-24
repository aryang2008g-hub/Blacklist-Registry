# 🚨 FraudRegistry — Stellar Soroban Smart Contract

> A decentralized, on-chain blocklist for flagging fraudulent wallet addresses across any application built on Stellar.

---

## Project Description

**FraudRegistry** is a permissioned smart contract deployed on the Stellar network using the **Soroban** smart-contract platform. It acts as a shared, tamper-resistant source of truth for known bad actors — phishing wallets, rug-pull deployers, scam accounts — that any dApp, exchange, or protocol on Stellar can query in real time.

Rather than every project maintaining its own private blocklist in a database, FraudRegistry externalises that trust layer onto the blockchain: transparent, auditable, and composable.

---

## What It Does

```
Reporter (trusted)         Any App / Protocol
      │                           │
      │  flag_address(addr, reason)│   is_flagged(addr)
      ▼                           ▼
┌─────────────────────────────────────────────────┐
│              FraudRegistry Contract              │
│                                                 │
│                        │
│  admin: Address                                 │
└─────────────────────────────────────────────────┘
```

1. **Admin** deploys and initialises the contract, then grants `reporter` roles to trusted entities (security researchers, community oracles, partner protocols).
2. **Reporters** call `flag_address()` with a target wallet and a reason code (`PHISHING`, `RUGPULL`, `SCAM`, etc.).
3. **Any on-chain or off-chain consumer** calls `is_flagged()` — a lightweight read — to gate transactions, warn users, or block interactions with the flagged address.
4. **Admin** can `clear_flag()` to handle false positives and `transfer_admin` for key rotation.

Each flag is stored as an immutable `FlagRecord` containing:
- The flagged address
- The reporter's address
- The reason string
- The ledger sequence (on-chain timestamp)
- An `active` boolean (cleared flags remain as audit history)

---

## Features

| Feature | Description |
|---|---|
| 🔐 **Role-based access** | Only admin-approved reporters can flag addresses; anyone can read |
| 📋 **Structured flag records** | Every flag stores reporter, reason, ledger timestamp, and active status |
| 🔍 **Composable reads** | `is_flagged()` is a simple boolean — easy to integrate into any contract or frontend |
| 🛡️ **False-positive handling** | Admin can deactivate flags without deleting the audit trail |
| 📜 **Full audit history** | Cleared flags remain on-chain with `active: false` for transparency |
| 🔄 **Admin key rotation** | `transfer_admin` allows safe handover to a multisig or DAO |
| ✅ **Test suite included** | Unit tests for flag, query, clear, and unauthorized-reporter scenarios |
| 🌐 **Cross-app compatible** | Any dApp on Stellar can call this contract — no SDK lock-in |

---

## Contract Interface

```rust
// Setup
fn initialize(env, admin: Address)

// Reporter management (admin only)
fn add_reporter(env, reporter: Address)
fn remove_reporter(env, reporter: Address)
fn is_reporter(env, addr: Address) -> bool

// Flagging (reporters only)
fn flag_address(env, reporter: Address, target: Address, reason: String)

// Querying (public)
fn is_flagged(env, addr: Address) -> bool
fn get_flag(env, addr: Address) -> FlagRecord

// Moderation (admin only)
fn clear_flag(env, addr: Address)
fn transfer_admin(env, new_admin: Address)
fn get_admin(env) -> Address
```

---

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) + `wasm32-unknown-unknown` target
- [Stellar CLI](https://developers.stellar.org/docs/tools/developer-tools/cli/install-and-setup)

```bash
rustup target add wasm32-unknown-unknown
cargo install --locked stellar-cli --features opt
```

### Build

```bash
cargo build --release --target wasm32-unknown-unknown
```

### Run Tests

```bash
cargo test
```

### Deploy to Testnet

```bash
# 1. Generate a keypair and fund it
stellar keys generate --global deployer --network testnet
stellar keys fund deployer --network testnet

# 2. Deploy the contract
stellar contract deploy \
  --wasm target/wasm32-unknown-unknown/release/fraud_registry.wasm \
  --source deployer \
  --network testnet

# 3. Initialize (replace CONTRACT_ID and ADMIN_ADDRESS)
stellar contract invoke \
  --id CONTRACT_ID \
  --source deployer \
  --network testnet \
  -- initialize \
  --admin ADMIN_ADDRESS
```

### Example Invocations

```bash
# Add a reporter
stellar contract invoke --id CONTRACT_ID --source admin --network testnet \
  -- add_reporter --reporter REPORTER_ADDRESS

# Flag an address
stellar contract invoke --id CONTRACT_ID --source reporter --network testnet \
  -- flag_address \
  --reporter REPORTER_ADDRESS \
  --target BAD_WALLET_ADDRESS \
  --reason '"PHISHING"'

# Check if flagged (read-only, free)
stellar contract invoke --id CONTRACT_ID --network testnet \
  -- is_flagged --addr WALLET_ADDRESS
```

---

## Use Cases

- **DEX / AMM** — Block swaps involving known scam wallets at the contract level
- **NFT Marketplace** — Warn buyers when a seller's address is flagged
- **Wallet UIs** — Display a warning badge before signing a transaction to a flagged address
- **Bridge protocols** — Refuse cross-chain transfers to blocklisted destinations
- **Community DAO** — Let token holders vote reporters in/out via the admin role

---

## Security Considerations

- **Reporter trust** is the main attack surface. Use a multisig or DAO address as admin and vet reporters carefully.
- **Reason strings** are stored on-chain; keep them short and use standardised codes to save ledger storage fees.
- Cleared flags are never deleted — the history is always visible. This is intentional for accountability.

---

wallet addres: GA2QZFO5TTPTUQJM64YMYVJKKISHHPTHKIQERE6EMERAE3LGCQ4BERS6

contract addres: CCXNY4LKLHT4HCQYKHCR4ZYMJ7POCOJ7KNPNYW2BAFO2G7KYWA6OFQGM

https://stellar.expert/explorer/testnet/contract/CCXNY4LKLHT4HCQYKHCR4ZYMJ7POCOJ7KNPNYW2BAFO2G7KYWA6OFQGM

<img width="1361" height="625" alt="image" src="https://github.com/user-attachments/assets/fb2424f6-05e1-4593-bfd2-91b5d5843073" />
