# Rollback & Destructive Safety Protocol

flareTUI enforces defensive safety confirmation dialogs for all irreversible operations.

## Guarded Operations

- **DNS Record Deletion**: Requires explicit confirmation (`y`/`n`).
- **Edge Cache Purge**: Displays zone-wide cache purge warnings before dispatching API mutations.
- **Pages Deployment Rollback**: Requires explicit target version verification before re-routing production traffic.
