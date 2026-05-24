# SPEL IDL Generation

## Two IDL Files

This solution ships **two** IDL files at `artifacts/`:

- `artifacts/token.idl.json` — hand-authored canonical IDL conforming to the `SpelIdl` schema. Covers all 11 instruction variants (original + authority), 3 account types, 4 custom types, and 2 error codes. **Completeness** — includes shapes that `spel generate-idl` does not emit (account data types, error table).

- `artifacts/token_authority.idl.spel.json` — `spel -- generate-idl` output from the `spel-sidecar/` scaffold. **Provenance** — proves the program shape parses through SPEL's macro grammar.

## Why a Sidecar

The straightforward path — annotate the real token program guest with `#[lez_program]` — does not compile. The reason is a dep-graph collision between two versions of `nssa_core`:

- `spel-framework v0.4.0` depends on `nssa_core` from `logos-execution-zone` tag `v0.2.0-rc3`
- Our workspace depends on `nssa_core` from `logos-execution-zone` rev `006647b` (HEAD)

The `spel_framework::prelude::AccountPostState` and `nssa_core::program::AccountPostState` become distinct types, causing type mismatches that cannot be resolved with `[patch]` without risking feature-flag and API drift.

## Reproducing the SPEL-Generated IDL

```bash
# Install SPEL CLI (one-time)
cargo install --git https://github.com/logos-co/spel --tag v0.4.0 spel

# Generate IDL from the sidecar scaffold
cd spel-sidecar
spel -- generate-idl src/lib.rs > ../artifacts/token_authority.idl.spel.json
```

The sidecar source at `spel-sidecar/src/lib.rs` mirrors the four authority instructions using SPEL macros (`#[lez_program]`, `#[instruction]`, `#[account(...)]`). It is not the real program — just a shape scaffold for IDL generation.
