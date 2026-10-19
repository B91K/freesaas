# Contributing

Thanks for helping build FreeSaaS!

## Getting started

```bash
git clone --recursive https://github.com/B91K/freesaas
cd freesaas
cargo test --workspace
cargo run --bin freesaas -- examples/local-dev.yaml
```

Before opening a pull request, run:

```bash
cargo fmt --all
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

## Where things go

- **Editor bugs and features** (WordCraft, GridCraft, DeckCraft) belong
  upstream at [github.com/storytold](https://github.com/storytold). Don't
  edit `upstream/` here.
- **New storage or identity providers** are the easiest way to contribute.
  See "Writing a provider" in [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

## Contributor License Agreement

FreeSaaS is free for individuals, non-profits and education, and
commercially licensed to companies (see [docs/LICENSING.md](docs/LICENSING.md)).
To keep that possible, contributors sign a CLA. You keep the copyright to
your work and grant the project the right to license it under both the BSL
and commercial terms. A CLA bot will be added before the first public
release.
