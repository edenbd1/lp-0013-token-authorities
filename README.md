# LP-0013: Token Program Improvements — Authorities

A rotatable mint authority model for the Logos Execution Zone (LEZ) Token program.

Set a mint authority at token creation, mint gated by that authority, rotate control to a new admin, or permanently revoke — enabling fixed-supply, variable-supply, and governance-handoff token patterns.

> Submission for [LP-0013 on ns.com](https://ns.com/earn/lp-0013-token-program-improvements-authorities). For an evaluator's checklist, see [`docs/criteria-checklist.md`](./docs/criteria-checklist.md).

## Quickstart

```bash
# Build everything.
cargo build --release --workspace

# Run all 76 tests (13 lez-approval + 55 token-authority-program + 6 integration + 2 SDK).
cargo test --workspace --release

# Run the fixed-supply example.
cargo run -p example-fixed-supply

# Run the variable-supply example.
cargo run -p example-variable-supply
```

## Architecture

Four new instruction variants layered additively on the existing Token program — pre-existing code paths are untouched:

```
┌─────────────────────────────────────┐
│       lez-approval (RFP-001)        │
│  Authority · gate · rotate · revoke │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│      token-authority-core           │
│  Instruction (11 variants)          │
│  TokenDefinition::FungibleWithAuth  │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│     token-authority-program         │
│  authority.rs · burn.rs · mint.rs   │
│  55 unit tests                      │
└──────────────┬──────────────────────┘
               │
┌──────────────▼──────────────────────┐
│      token-authority-sdk            │
│  Type re-exports · TX docs          │
└─────────────────────────────────────┘
```

### New Instructions

| Instruction | Accounts | Description |
|---|---|---|
| `NewFungibleDefinitionWithAuthority` | definition (auth), holding (auth) | Create token with mint authority |
| `MintWithAuthority` | definition (auth), holding, authority_signer (auth) | Authority-gated minting |
| `RotateAuthority` | definition (auth), authority_signer (auth) | Atomic authority rotation |
| `RevokeAuthority` | definition (auth), authority_signer (auth) | Permanent revocation |

### Security Model

Authority is verified **cryptographically** — the signer is a pre_state account with `is_authorized` checked by the NSSA runtime. Not instruction data. See [`docs/security.md`](docs/security.md).

### Backward Compatibility

- `FungibleWithAuthority` is a **new enum variant** — existing `Fungible` definitions are unchanged
- Regular `Mint` is rejected on authority tokens (must use `MintWithAuthority`)
- `Burn` works normally on authority tokens

## Token Patterns

### Fixed Supply
```
new-with-authority → revoke-authority
```
Mint the initial supply, then permanently revoke. Supply is provably fixed.

### Variable Supply
```
new-with-authority → mint-with-authority → mint-with-authority → ...
```
Authority can mint tokens as needed.

### Governance Handoff
```
new-with-authority → rotate-authority → (new admin takes over)
```
Transfer mint control to a DAO, multisig, or successor.

## Repository Structure

```
crates/
  lez-approval/              RFP-001 agnostic approval library (13 tests)
  token-authority-core/       Extended instruction & definition types
  token-authority-program/    Handler implementations (55 tests)
  token-authority-sdk/        Type re-exports and TX construction docs
examples/
  fixed-supply/               Mint-then-revoke pattern
  variable-supply/            Rotatable authority pattern
integration_tests/            6 end-to-end handler-pipeline tests
artifacts/
  token.idl.json              Canonical IDL (SpelIdl format)
scripts/
  demo.sh                     End-to-end lifecycle demo
docs/
  criteria-checklist.md       Row-by-row LP-0013 criteria mapping
  architecture.md             Component diagram and security model
  design.md                   Design decisions with alternatives
  error-codes.md              Complete error reference
  security.md                 Threat model and Logos stack properties
  benchmarks/cu-budget.md     Compute unit overhead analysis
```

## LEZ Fork Integration

The standalone crates in this repo are integrated into the actual LEZ token program at [`edenbd1/logos-execution-zone`](https://github.com/edenbd1/logos-execution-zone/tree/lp-0013-token-authorities) (branch `lp-0013-token-authorities`). That fork includes wallet CLI subcommands (`new-with-authority`, `mint-with-authority`, `rotate-authority`, `revoke-authority`) and the guest binary dispatcher.

## Testing

```bash
# All 76 tests.
cargo test --workspace --release

# Just the approval library.
cargo test -p lez-approval

# Just the token program (55 tests including authority).
cargo test -p token-authority-program

# Integration tests (full handler pipeline).
cargo test -p integration-tests
```

## Documentation

| Document | Description |
|---|---|
| [`docs/criteria-checklist.md`](docs/criteria-checklist.md) | Every LP-0013 criterion → evidence |
| [`docs/architecture.md`](docs/architecture.md) | Components, state machine, security |
| [`docs/design.md`](docs/design.md) | Decisions and alternatives |
| [`docs/error-codes.md`](docs/error-codes.md) | Error reference |
| [`docs/security.md`](docs/security.md) | Threat model |
| [`docs/benchmarks/cu-budget.md`](docs/benchmarks/cu-budget.md) | CU overhead analysis |
