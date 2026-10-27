# Urja Meter Ops — Upstream Protocol Notes

## Overview

Urja Meter Ops is a web portal for looking up electricity meters, their geographic location, their network hierarchy, and their energy data.

The API service communicates with the portal over HTTPS using the endpoints observed in the browser's Network tab and frontend JavaScript.

The portal uses authenticated sessions and, for the bulk export endpoint, request signing.

## Authentication and session handling

### Login

The portal accepts a form-encoded login request:

```http
POST /login
Content-Type: application/x-www-form-urlencoded
x-sveltekit-action: true
```

Form fields:

- `email`
- `password`

A successful login sets a session cookie named:

`__Secure-better-auth.session_token`

The Rust client uses a reusable Reqwest client configured with a cookie jar. This allows the session cookie to be retained and sent automatically on subsequent requests.

Credentials and session cookies must not be committed to source control or written to logs.

### Session considerations

The session cookie observed in the browser has a limited lifetime. Session expiration and re-authentication should be handled explicitly for a robust long-running service.

The exact expiration and renewal behavior should be confirmed through testing rather than assumed.

## Discovered endpoints

### 1. Meter search

```http
GET /portal/meters/search?q={query}&page={page}
```

Returns paginated meter search results.

An observed response has a structure similar to:

```json
{
  "data": {
    "data": [
      {
        "dtCode": "DT-001",
        "installStatus": "Decommissioned",
        "make": "HPL",
        "meterId": "J100000",
        "phaseType": "single",
        "serialNo": "SE33962"
      }
    ],
    "page": 1,
    "pageSize": 20,
    "total": 403
  }
}
```

The example illustrates the observed response structure; the actual records and totals may change.

### 2. Meter geographic details

```http
GET /portal/meters/{meterId}/geo
```

Returns geographic coordinates for an individual meter.

An observed response:

```json
{
  "data": {
    "latitude": "26.938961002479868",
    "longitude": "75.83095696146852"
  }
}
```

The coordinate values in this response were represented as strings.

### 3. Meter energy details

```http
GET /portal/meters/{meterId}/energy
```

Returns energy-related data for an individual meter. The response is JSON.

The exact energy response schema should be documented from a representative captured response if consumers need a stable, field-by-field contract.

### 4. Distribution transformers

```http
GET /portal/dts?page={page}
```

Returns paginated distribution-transformer data. The observed frontend displays transformer code, name, feeder code, and capacity in kVA.

### 5. Signing key

```http
GET /portal/keys
```

Returns a JSON object containing a signing secret:

```json
{
  "data": {
    "signingSecret": "<redacted>"
  }
}
```

The value is sensitive and must not be logged, hardcoded, or committed to source control.

### 6. Bulk meter export

```http
GET /portal/export?page={page}
```

This endpoint requires the authenticated session and signed request headers:

```http
x-timestamp: <unix-timestamp-seconds>
x-signature: <hex-encoded-signature>
```

The browser frontend first requests `/portal/keys`, then generates a signature before calling `/portal/export`.

## Request-signing algorithm

Inspection of the frontend JavaScript revealed the following signing process:

1. Obtain the signing secret from `/portal/keys`.
2. Generate the current Unix timestamp in seconds.
3. Construct the message by joining four values with newline characters.
4. Compute an HMAC using SHA-256.
5. Encode the resulting digest as lowercase hexadecimal.
6. Send the timestamp and signature in the request headers.

The message format is:

```text
HTTP_METHOD
PATH
QUERY_STRING
TIMESTAMP
```

For example, for a first-page export:

```text
GET
/portal/export
page=1
<TIMESTAMP>
```

The actual timestamp is inserted without angle brackets. The query string is `page=1`, without a leading question mark.

Equivalent pseudocode:

```text
timestamp = unix_time_in_seconds()
message = method + "\n" + path + "\n" + query + "\n" + timestamp
signature = hex(HMAC_SHA256(signing_secret, message))
```

The signature must be generated for each request using the current timestamp. Reusing a previously captured signature or timestamp is not a reliable approach.

When adding query parameters, the query string used for signing must match the query string sent in the request.

## Exported data and network hierarchy

The export returns meter records containing fields such as:

- `meterId`
- `serialNo`
- `make`
- `phaseType`
- `installStatus`
- `installType`
- `build`
- `dtCode`
- `hierarchy`
- `geo`

The hierarchy object can include:

- Zone
- Circle
- Division
- Subdivision
- Substation
- Feeder
- Distribution transformer (`dt`)

Each hierarchy level contains a name and code in the observed export format.

The geographic object uses numeric `lat` and `lng` values in the export, unlike the string-valued coordinates observed in the individual meter geo endpoint.

The export is useful for reconstructing network relationships and building a local query index. Its pagination and dataset coverage should be verified before assuming that one response contains every meter.

## Observed quirks and implementation considerations

- The portal is designed for browser use, but its underlying endpoints can be called over HTTPS.
- Login state is maintained using a session cookie.
- The export endpoint requires request-signing headers; ordinary session authentication alone resulted in an unauthorized response during exploration.
- The frontend signs the HTTP method, path, query string, and timestamp separated by newline characters.
- The signing secret is obtained dynamically from `/portal/keys`.
- Data formats differ between endpoints; for example, coordinates are strings in the observed geo response and numbers in the export.
- The service should not assume upstream schemas remain stable indefinitely.
- Upstream errors should be translated into clear API errors without exposing secrets or unnecessary internal details.

## Evidence and scope

These notes are based on observed browser Network requests, captured JSON responses, and inspection of the frontend signing function.

Where behavior was not fully verified—such as session renewal, exhaustive export pagination, or the precise energy schema—it is intentionally identified as an open question rather than treated as a confirmed guarantee.

