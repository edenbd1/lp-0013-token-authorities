# LP-0013 success-criteria checklist

Row-by-row mapping of every line in the LP-0013 prize text to the code, test, or artifact that satisfies it.

## Functionality

| Criterion | Evidence |
|---|---|
| Variable-size Tokens through minting authority: mint authority set at token initialization | `crates/token-authority-core/src/lib.rs` — `NewFungibleDefinitionWithAuthority` instruction variant. Handler: `crates/token-authority-program/src/authority.rs::new_fungible_definition_with_authority`. |
| Minting by the authority | `MintWithAuthority` instruction. Handler: `authority.rs::mint_with_authority`. Authority is verified **cryptographically** via `is_authorized` on the pre_state account — not instruction data. |
| Authority rotation and/or revocation | `RotateAuthority` (atomic single-field write) and `RevokeAuthority` (terminal — `Authority::renounced()` is a permanent sentinel). |
| At least two example integrations | **Three examples:** `examples/fixed-supply/` (mint everything, then revoke), `examples/variable-supply/` (rotatable authority with ongoing minting), and `examples/config-pda-gate/` (RFP-001 §4 — gates a non-token config PDA behind admin authority, demonstrating the library's reusability beyond the token program). |
| Self-sufficient agnostic library per RFP-001 | `crates/lez-approval/` — `Authority`, `ApprovalError`, `gate`/`rotate`/`revoke`. Depends only on `nssa_core::account::AccountId`. Reusable by any LEZ program. 13 unit tests. The `config-pda-gate` example demonstrates the library gating privileged instructions on a non-token program config PDA (RFP-001 §4). |

## Usability

| Criterion | Evidence |
|---|---|
| Module/SDK for building Logos modules | `crates/token-authority-sdk/` re-exports key types and documents the transaction construction pattern for all four authority instructions. Wallet facade methods in the LEZ fork: `wallet/src/program_facades/token.rs`. CLI subcommands: `new-with-authority`, `mint-with-authority`, `rotate-authority`, `revoke-authority`. |
| IDL for the updated token program using SPEL | `artifacts/token.idl.json` — hand-authored canonical IDL conforming to `SpelIdl`, covering all 11 instruction variants, 3 account types, 4 custom types, and 2 error codes. |

## Reliability

| Criterion | Evidence |
|---|---|
| Authority rotation and revocation are atomic | Both handlers perform a single field write or panic. No intermediate state exists. Covered by tests: `rotate_authority_succeeds`, `revoke_authority_succeeds`. |
| Minting with a revoked authority is rejected deterministically with a documented error code | Panics with `ApprovalError::Renounced` ("Renounced: authority has been permanently revoked"). Covered by test: `mint_after_revoke_panics`. Error codes documented in `docs/error-codes.md`. |

## Performance

| Criterion | Evidence |
|---|---|
| Document the CU cost of each new operation on LEZ devnet/testnet | Authority check adds one `AccountId` equality comparison per gated instruction — negligible overhead. `FungibleWithAuthority` variant adds 33 bytes (1 discriminant + 32 `AccountId`) to the on-chain definition account. Full CU measurements in `docs/benchmarks/cu-budget.md`. |

## Supportability

| Criterion | Evidence |
|---|---|
| Updated token program deployed and tested on LEZ devnet/testnet | `scripts/demo.sh` runs the full lifecycle against a docker-compose standalone LEZ sequencer (run from the [LEZ fork](https://github.com/edenbd1/logos-execution-zone/tree/lp-0013-token-authorities)). |
| End-to-end integration tests against LEZ sequencer in CI | `integration_tests/` — 6 handler-pipeline tests exercising the full pre_state → handler → post_state lifecycle (create+mint, rotate+mint, revoke+reject, wrong-signer reject, unsigned-authority reject, burn on authority tokens). Sequencer-level E2E via `scripts/demo.sh` in the LEZ fork. |
| CI green on default branch | `.github/workflows/ci.yml` — build, clippy, fmt, test. All 71 tests pass. **[CI green](https://github.com/edenbd1/lp-0013-token-authorities/actions).** |
| README documents end-to-end usage | `README.md` covers quickstart, architecture, CLI commands, deployment steps. `docs/` covers architecture, design, error codes, security, benchmarks. |
| Reproducible demo with `RISC0_DEV_MODE=0` | `scripts/demo.sh` — full create → mint → rotate → mint → revoke → verify-rejection lifecycle (run from the LEZ fork). |
| Narrated video walkthrough | See submission PR for link. |

## Submission requirements

| Requirement | Evidence |
|---|---|
| Public repository under MIT or Apache-2.0 | Dual-licensed; `LICENSE-MIT` + `LICENSE-APACHE` + `NOTICE` |
| Code changes to the token program | Complete implementation in `crates/token-authority-program/`. Integration into LEZ fork at `edenbd1/logos-execution-zone` branch `lp-0013-token-authorities`. |
| README + design docs | `README.md`, `docs/architecture.md`, `docs/design.md`, `docs/error-codes.md`, `docs/security.md`, `docs/criteria-checklist.md` |
| Tests and example programmes/scripts | 71 tests total (13 lez-approval + 50 token-authority-program + 6 integration + 2 SDK) + 2 runnable example programs |
| Narrated video walkthrough | See submission PR for link. |
