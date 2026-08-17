# Authentication

flareTUI offers two secure authentication mechanisms: automatic browser-based OAuth with PKCE and manual API Token entry.

## 1. Browser Login (OAuth)

When selecting **Browser Login**:
1. flareTUI binds a temporary local HTTP loopback server on `127.0.0.1`.
2. It automatically launches your system browser to Cloudflare's authorization portal.
3. Upon approval, the loopback server receives the authorization code, validates the PKCE state parameter, and acquires the access token.

## 2. API Token (Manual)

To manually supply a token, press `Ctrl+A` or configure `~/.config/flaretui/config.toml`.

### Required Token Scopes

| Category | Scope | Permissions |
| :--- | :--- | :--- |
| Zone | DNS | Edit |
| Zone | Zone Settings | Read / Edit |
| Account | Worker Scripts | Read |
| Account | Worker Tail | Read |
| Account | Pages | Read / Edit |
| Account | D1 | Read / Edit |
| Account | Workers KV Storage | Read |
| Zone | Security Events | Read |
| Zone | WAF Rulesets | Read / Edit |
| Account | Registrar Domains | Read / Edit |

## Profile Management

Switch between accounts seamlessly using `Ctrl+U` or clear current credentials using `Ctrl+L`.
