# View 2 & 5: Workers & Live Tail

flareTUI offers deep visibility into Cloudflare Workers serverless execution and real-time streaming logs.

## Workers Inspector (Tab 2)

- Browse all registered Worker scripts, deployment timestamps, and compatibility flags.
- Press `i` to inspect routes, custom domains, and attached bindings (D1 databases, KV namespaces, R2 buckets, Queues).
- Press `l` to immediately open a WebSocket Live Tail session for the selected script.

## Realtime Live Tail (Tab 5)

Stream execution logs directly from Cloudflare's edge network over WebSockets:

- `f`: Toggle Follow Mode (auto-scroll to newest incoming logs).
- `Space`: Pause / resume live log stream ingestion.
- `c`: Clear local log memory buffer.
- `Enter`: Open full JSON log inspector with exception trace decoding.
- `/`: Filter logs in-memory by HTTP method, status code, URL pattern, or error keyword.
