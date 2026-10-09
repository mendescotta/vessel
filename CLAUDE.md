# vessel

GTK4/Rust ISO profile builder: a validated `profile.toml` generates a
`build.sh` that builds a Void live ISO. Design notes: @DEVNOTES.md (local, gitignored).

## Check before claiming done
- `cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test`
- Golden tests: `tests/golden/*.sh` must match generator output. Regenerate
  only deliberately with `VESSEL_BLESS=1 cargo test` and review the diff.

## Rules
- `profile::validate` is the only way to build a `ValidProfile`; keep
  `generate` infallible. Reject argument values starting with `-`.
- Generated scripts use a private `/dev`, wipe only folders marked
  `.vessel-work`, and write files via temp file + rename.
- Targets: `x86_64` and `x86_64-musl`; userland is GNU only.
- Builds that run as root (`pkexec`, `build.sh`) are not run by Claude
  without being asked.
