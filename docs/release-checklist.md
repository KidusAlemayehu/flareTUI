# Release Checklist

1. Run full test suite: `cargo test`.
2. Verify all 7 views render properly in 80x24 and ultrawide terminals.
3. Bump version in `Cargo.toml`.
4. Synchronize release notes in `CHANGELOG.md`.
5. Create signed Git release tag: `git tag -a vX.Y.Z -m "Release vX.Y.Z"`.
6. Publish release binary artifacts.
