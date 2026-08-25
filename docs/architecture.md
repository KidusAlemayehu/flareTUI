# Engine Architecture

flareTUI is built using an asynchronous, action-driven architecture separating terminal presentation from network I/O.

## High-Level Pipeline

1. **Terminal Presentation**: Driven by Ratatui and Crossterm, running on an alternate screen with raw mode enabled.
2. **Event Ingestion**: A dedicated thread polls Crossterm keyboard events and 200ms periodic ticks, transmitting them to the central event channel.
3. **Action Queue**: An unbounded MPSC channel processes actions asynchronously (API responses, WebSocket packets, state mutations).
4. **Offline Cache**: Embedded SQLite database (`rusqlite`) serving as a local read-through and write-through cache for sub-millisecond retrieval.
5. **Network Client**: Reqwest client with HTTP/3 support communicating with Cloudflare v4 REST API and Tokio Tungstenite for WebSocket streams.
