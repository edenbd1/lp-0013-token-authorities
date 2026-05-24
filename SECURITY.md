# Security policy

## Reporting a vulnerability

If you find a security issue, please **do not** open a public GitHub issue.

Contact: eden.baudin.invest@gmail.com

We aim to acknowledge reports within 72 hours and provide a status update within 7 days.

## Scope

The following are in scope:

- `crates/lez-approval/` — LEZ approval logic.
- `crates/token-authority-core/` — authority model types and validation.
- `crates/token-authority-program/` — on-chain program logic.
- `crates/token-authority-sdk/` — host-side SDK for authority operations.
- `crates/token-authority-cli/` — CLI tool.

Out of scope:

- Issues in upstream dependencies.
- Issues in `_external/` (vendored reference repos; report upstream).

## Audit status

**Unaudited.** Treat this code as a reference implementation. Do not deploy to a value-bearing context without a third-party audit.
