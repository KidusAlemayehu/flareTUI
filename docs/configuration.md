# Configuration & Themes

Configuration is stored in `~/.config/flaretui/config.toml`.

## Example Configuration

```toml
current_profile = "production"
theme = "cloudflare-dark"

[settings]
fast_sync = true
sync_interval_seconds = 300

[[profiles]]
name = "production"
api_token = "cfut_prod_token_abcdef1234567890"
account_id = "0123456789abcdef0123456789abcdef"
account_name = "Production Team"

[[profiles]]
name = "staging"
api_token = "cfut_staging_token_1234567890abcdef"
account_id = "fedcba9876543210fedcba9876543210"
account_name = "Staging Environment"
```

## Available Themes

- `cloudflare-dark`: Official Cloudflare signature orange and dark navy.
- `catppuccin-mocha`: Soothing pastel palette with high contrast.
- `tokyo-night`: Modern cyber-aesthetic neon palette.
- `nord`: Arctic-inspired clean cool colors.
