# AGENTS.md

## Workflow

After **every** code change to Codify:

1. `cargo check`
2. `cargo test` (must stay green)
3. `cargo build --release`
4. Smoke-test the release binary: `timeout 20 ./target/release/codify /tmp/opencode; echo $?` (expect `124`)
5. `git add -A && git commit -m "<message>"`
6. `git push origin main`

Repo: https://github.com/lordpipon/codify (branch `main`).

## Notes

- No comments in code unless asked.
- No emojis in code/UI unless asked.
- Windows/macOS installers are built by CI (`.github/workflows/release.yml`); only the Linux release build is verified locally.
