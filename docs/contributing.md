# Contributing Guidelines

We welcome community contributions!

## Development Workflow

1. Fork and clone the repository.
2. Create a feature branch: `git checkout -b feature/my-feature`.
3. Verify test coverage: `cargo test`.
4. Ensure clean formatting and zero linter warnings:
   ```bash
   cargo fmt --check
   cargo clippy -- -D warnings
   ```
5. Submit a pull request.
