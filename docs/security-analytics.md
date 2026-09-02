# Security Analytics & Threat Scoring

Analyze real-time security events captured by Cloudflare WAF.

## Threat Score Metric

flareTUI calculates a composite threat score (0-100) based on:
- High-severity block actions vs total requests.
- Rate-limiting threshold triggers.
- Geographic threat concentration.

## Event Inspector

Press `Enter` on any security event to open detailed diagnostic records:
- Client IP address (press `c` to copy to clipboard).
- Cloudflare Ray ID (press `r` to copy to clipboard).
- Full matched WAF rule expression and trigger payload.
