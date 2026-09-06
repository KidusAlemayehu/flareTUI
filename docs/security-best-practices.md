# Security Best Practices

Guidelines for maintaining minimal privilege and protecting Cloudflare credentials.

## Recommended Practices

1. **Least Privilege**: Avoid Global API Keys. Use scoped User API Tokens restricted to specific zones.
2. **File Permissions**: Ensure `~/.config/flaretui/config.toml` has restricted read permissions (`chmod 600`).
3. **Session Cleansing**: Use `Ctrl+L` to log out active sessions on shared machines.
