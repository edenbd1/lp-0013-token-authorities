# Contributing

LP-0013 is a focused submission for a single λPrize. Contributions are welcome — please open an issue first to discuss scope.

## Development setup

```bash
# Standard Rust toolchain — no special guest toolchain needed.
rustup toolchain install stable
```

## Running tests

```bash
# Unit + integration tests.
cargo test --workspace --release

# Run the demo script.
./scripts/demo.sh
```

## Conventions

- One logical change per commit; commit messages explain *why*, not what.
- Public API changes update the relevant `crates/*/README.md`.
- New authority semantics ship with at least one negative test.
- No `Co-Authored-By` lines in commits.

## Reviewing pull requests

- Read [`docs/design.md`](./docs/design.md) before reviewing authority-model changes.
- Run the demo (`scripts/demo.sh`) before approving anything that touches the program or the SDK.
