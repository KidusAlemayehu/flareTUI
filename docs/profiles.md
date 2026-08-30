# Multi-Account Profiles

Manage multiple Cloudflare organizations, personal domains, and client accounts seamlessly.

## Switching Profiles

1. Press `Ctrl+U` to open the profile switcher dialog.
2. Select the target profile using `Up`/`Down` and press `Enter`.
3. flareTUI immediately switches active API credentials, refreshes local cache contexts, and updates the header bar.

## Security

Tokens are masked in the UI. To prevent accidental token leakage, only truncated prefixes are shown.
