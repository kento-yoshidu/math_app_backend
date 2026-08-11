# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project state

This is a fresh-start Rust project named `math_app`. The working tree has just been reset from an older, unrelated Diesel/PostgreSQL todo-API project (`RustWebApp`, still visible in git history) — that project's files (`diesel.toml`, `migrations/`, `src/db.rs`, `src/schema.rs`, `src/todos/`) have been deleted but the deletions are not yet committed. `Cargo.toml` currently declares **no dependencies**, and `src/main.rs` is a bare `Hello, world!` binary.

Do not assume Diesel, Postgres, or any web framework is present — always check `Cargo.toml` before referencing a dependency, since the project is being rebuilt from scratch under the new name and purpose (a math app).

## Commands

- Build: `cargo build`
- Run: `cargo run`
- Check without building: `cargo check`
- Test: `cargo test` (no tests exist yet)
