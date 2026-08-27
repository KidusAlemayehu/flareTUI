# Troubleshooting & FAQ

## Common Issues

### 1. HTTP 403 Forbidden / Invalid Token
Ensure your Cloudflare API token is generated under **User Profile -> API Tokens** as a **User API Token**, and has permissions for the specific zones and accounts you are accessing.

### 2. OAuth Callback Port Conflict
If port 8080 is occupied, flareTUI automatically falls back to secondary local loopback ports (8081, 8082). If all ports are blocked, use manual API token entry.

### 3. Terminal Garbled After Exit
If an unexpected termination occurs, run `reset` in your shell to restore the standard screen buffer and cursor visibility. flareTUI installs panic hooks to automate this under normal crash conditions.
