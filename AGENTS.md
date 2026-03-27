# Repository Guidelines

## Project Structure & Module Organization
`src/` contains the Rust application code. `main.rs` wires the Actix server and CLI entrypoint, `lib.rs` re-exports shared modules, `cli.rs` and `daemon.rs` handle process control, `db/` contains SQLx-backed models and repository logic, and `handlers/` contains HTTP page and API handlers. UI assets live in `public/` and HTML templates live in `templates/`. Integration and module tests live in `tests/`, with one file per area such as `api_integration_test.rs` and `repository_test.rs`.

## Build, Test, and Development Commands
Use `cargo run -- start --foreground` to run the server locally without daemonizing. Use `cargo build --release` for a production binary. Run `cargo test` before opening a PR; this project uses async integration tests heavily. Keep formatting and lint checks clean with `cargo fmt -- --check` and `cargo clippy -- -D warnings`. If you use auto-reload locally, `cargo watch -x run` is documented in the repo README.

## Coding Style & Naming Conventions
Follow standard Rust formatting with `rustfmt`; use 4-space indentation and keep imports grouped logically. Prefer `snake_case` for files, modules, functions, and test names, and `CamelCase` for types and enums. Keep handlers focused on request/response logic and push persistence into `db/repository.rs`. Reuse constants from `src/config.rs` for limits such as message, reply, and search lengths instead of duplicating literal values.

## Testing Guidelines
Tests use Rust’s built-in test framework plus `actix_rt::test` for async cases. Add new tests in `tests/` when changing HTTP behavior, repository queries, CLI behavior, static assets, or security-sensitive validation. Name tests descriptively, for example `test_api_messages_with_params`. Prefer in-memory SQLite (`sqlite::memory:`) for repository and handler tests.

## Commit & Pull Request Guidelines
Recent history uses short, imperative commit subjects such as `Add --host parameter support`, `Fix issues from code review`, and `Bump version to 1.3.4`. Keep commits focused and use that style. PRs should describe user-visible behavior, note any config or CLI changes, and list the commands you ran locally. Include screenshots when changing `templates/` or `public/` output, and confirm CI-equivalent checks pass before requesting review.

## Configuration & Security Tips
Default runtime settings come from `HOST`, `PORT`, `DATA_DIR`, and `DATABASE_URL`. Keep local data under the default `~/.message-board` unless a test needs isolation. Treat `0.0.0.0` and `::` bindings as intentional exposure and document that choice in reviews.
