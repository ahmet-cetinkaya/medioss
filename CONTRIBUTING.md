# Contributing to MediOSS

Thanks for helping. By contributing you agree that your work is licensed under [AGPL-3.0-or-later](LICENSE). Please follow the [Code of Conduct](CODE_OF_CONDUCT.md). Report security problems privately, see [SECURITY.md](SECURITY.md).

## Before you start

- Read [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md). Modules never depend on each other, and layers point downward only.
- The UI follows [`docs/DESIGN_SYSTEM.md`](docs/DESIGN_SYSTEM.md).
- Feature work is specified under `specs/NNNN-<name>/` (what and why) with a `PLAN.md` (decisions and tasks). For a larger change, open an issue first so we can agree on the behavior.

## Set up and check

Every workflow goes through the `Makefile`:

```bash
make run      # build the UI and serve API + UI on :3000
make lint     # prettier, cargo fmt, clippy -D warnings, boundary check
make test     # bun test, cargo test --workspace, boundary-check self-test
```

You need Rust (version pinned in `rust-toolchain.toml`), bun, and `ffmpeg` on PATH. Always run cargo with `--workspace` or `-p`.

## Pull requests

- Keep a PR to one topic. Add or update tests with the change.
- Make sure `make lint` and `make test` pass; CI runs the same gates.
- Use Conventional Commit messages (`feat:`, `fix:`, `refactor:`, `docs:`, `chore:`).
- Describe what changed and why, and how you verified it.
