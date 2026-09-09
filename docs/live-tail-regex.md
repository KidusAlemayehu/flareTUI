# Live Tail Regex Cheatsheet

Filter real-time Worker logs using standard regex patterns (`/`).

## Example Filters

- Filter by HTTP 5xx errors: `status:5\d\d` or `500`
- Filter by API route: `/api/v[0-9]+/users`
- Filter by unhandled exception: `Error|Exception|panic`
- Filter by HTTP method: `POST|PUT|DELETE`
