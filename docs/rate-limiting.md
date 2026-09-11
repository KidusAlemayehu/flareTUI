# API Rate Limiting & Backoff

Cloudflare imposes rate limits of 1,200 requests per 5 minutes per user token.

## flareTUI Handling

- **Request Throttling**: Background sync queries are staggered to prevent burst limits.
- **Exponential Backoff**: HTTP 429 responses trigger exponential backoff with jitter.
- **Local Cache Resiliency**: When rate limits are reached, flareTUI serves from the local SQLite cache without interrupting user navigation.
