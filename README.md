# Flock Energy — Meter Operations API

A Rust API service built with Axum and Reqwest that exposes meter, energy, geographic, distribution-transformer, and bulk-export data from the legacy Urja Meter Ops portal.

## Features

- Authenticate with the upstream portal and manage its session cookies.
- Search and paginate meter records.
- Retrieve geographic coordinates for an individual meter.
- Retrieve energy data for an individual meter.
- List distribution transformers.
- Generate signed export requests using HMAC-SHA256.
- Export meter data using the portal's signed bulk-export endpoint.

## Tech stack

- **Rust** — implementation language.
- **Axum** — HTTP API and routing.
- **Reqwest** — upstream HTTP client and cookie handling.
- **Serde / Serde JSON** — JSON serialization and deserialization.
- **HMAC-SHA256** — signing requests to the upstream export endpoint.

## Architecture

The service acts as an API layer over the existing portal.

1. Load configuration from environment variables.
2. Create a reusable HTTP client with cookie storage.
3. Authenticate with the portal.
4. Receive requests through Axum.
5. Forward requests to the relevant portal endpoint.
6. Return JSON responses and translate upstream failures into API errors.

The export endpoint first retrieves the portal's signing secret, computes the required request signature, and then requests the bulk export.

See [PROTOCOL.md](PROTOCOL.md) for details of the upstream authentication and signing flow.

## Requirements

- Rust toolchain and Cargo.
- Network access to the Urja Meter Ops portal.
- Valid portal credentials.

## Configuration

Create a `.env` file in the project root:

```dotenv
PORTAL_BASE_URL=https://urja-ops.flockenergy.tech
PORTAL_EMAIL=your-portal-email
PORTAL_PASSWORD=your-portal-password
PORT=8080

## Run locally

```bash
git clone <your-public-repository-url>
cd flock-energy-api

cp .env.example .env
# Edit .env with your portal credentials.

cargo run
```

The server listens on the port configured by `PORT`. If your application defaults to a different port, use that value instead.

Check that the API is running:

```bash
curl -i http://127.0.0.1:8080/health
```

or use Postman or similar services.

## API endpoints

| Method | Endpoint | Description |
|---|---|---|
| GET | `/health` | Health check |
| GET | `/meters?q=J100000&page=1` | Search and paginate meters |
| GET | `/meters/{id}/geo` | Retrieve meter coordinates |
| GET | `/meters/{id}/energy` | Retrieve meter energy data |
| GET | `/dts?page=1` | List distribution transformers |
| GET | `/export?page=1` | Retrieve signed bulk meter export |

### Examples

Health check:

```bash
curl http://127.0.0.1:8080/health
```

Search meters:

```bash
curl -G http://127.0.0.1:8080/meters \
  --data-urlencode 'q=J100000' \
  --data-urlencode 'page=1'
```

Get meter coordinates:

```bash
curl http://127.0.0.1:8080/meters/J100000/geo
```

Get meter energy data:

```bash
curl http://127.0.0.1:8080/meters/J100000/energy
```

List distribution transformers:

```bash
curl 'http://127.0.0.1:8080/dts?page=1'
```

Export meter data:

```bash
curl 'http://127.0.0.1:8080/export?page=1'
```

See [`openapi.json`](openapi.json) for the API specification.

## Error handling

The API validates supported query parameters where implemented and returns JSON errors for failures. Upstream portal failures should be surfaced as gateway errors rather than presented as successful API responses.

The current implementation may include upstream error details during development. These should be kept out of public production responses because they can expose implementation details or sensitive information.

## Assumptions and design decisions

- The upstream portal remains the source of truth; this service does not modify portal data.
- JSON responses are retained where practical to avoid losing upstream fields.
- A reusable Reqwest client manages the authenticated session and cookies.
- The export endpoint uses the portal's documented-by-observation signing flow rather than a hardcoded signature.
- The service does not persist portal data locally or maintain a database.
- Upstream data shapes and behavior may change because the portal is a legacy system.

## Intentional limitations

- No local database, persistent cache, or search index.
- No write operations against the portal.
- No separate API-key or user-authentication layer for consumers unless added separately.
- No dedicated network-hierarchy query endpoints in the current version.
- No automatic retry or session re-authentication strategy unless.

## Future improvements

- Add typed response models instead of passing arbitrary JSON for every endpoint.
- Add timeouts and a deliberate retry policy for transient upstream failures.
- Handle session expiry and re-authentication gracefully.
- Add caching with explicit freshness rules where appropriate.
- Build a local index from the bulk export to support hierarchy queries and cross-meter filtering.
- Add structured logging, metrics, and request tracing.
- Validate the OpenAPI specification against actual responses.

## Reflection

### What assumptions did you make?

I treated the portal as the source of truth and built a read-only API over the endpoints available to an authenticated user. I preserved the upstream JSON shape where practical to avoid discarding fields that downstream consumers might need.

### Which part was the most difficult, and how did you get unstuck?

The export endpoint required more than a normal authenticated HTTP request. By inspecting the browser's Network tab and the frontend bundle, I found that the portal retrieves a signing secret and uses HMAC-SHA256 to sign the method, path, query string, and timestamp. Reproducing that request flow made the export work from the Rust client.

### If you had another day, what would you improve?

I would add integration tests, typed response models, robust session-expiry handling, and a local index built from the bulk export. The index would make network-hierarchy queries and filtering across meter attributes more efficient.

### What mistake did you make while solving this?

I initially treated the export as a conventional authenticated endpoint and did not account for its signature headers. Inspecting the browser request and frontend implementation exposed the additional signing step.

### If you were reviewing your own submission, what would you criticise?
The service currently depends on the upstream portal's availability and response formats. I would want stronger automated tests, explicit timeout and retry policies, and verified behavior for session expiry before considering it production-ready.

