# Performance Benchmarks

flareTUI is optimized for instant startup and low resource consumption.

## Metrics

- **Cold Boot Time**: < 15ms (via SQLite cached state).
- **Memory Footprint**: < 25MB RSS under continuous live tail ingestion.
- **Binary Size**: ~18MB release binary with full debug symbols stripped.
- **Frame Render Time**: < 1.5ms per frame on standard modern terminal emulators.
