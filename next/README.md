# memory.lol: next

A modernized replacement for the `web/` service and `app/` frontend, intended to
supplant them once deployed. Three projects:

| Project           | Replaces    | Stack                                            |
| ----------------- | ----------- | ------------------------------------------------ |
| `auth-rusqlite/`  | `auth-sqlx` | `rusqlite` (via `tokio-rusqlite`)                |
| `api/`            | `web/`      | Axum, `tower-http`, `figment`                    |
| `app/`            | `app/`      | Vite, SvelteKit (SPA), Svelte 5, Tailwind CSS 4  |

This directory is a separate Cargo workspace: modern `rusqlite` and the legacy
stack's `sqlx-sqlite` both link the native `sqlite3` library at incompatible
versions, so they cannot share one dependency graph. When `web/` and `auth-sqlx`
are retired, these crates can be folded into the root workspace.

## API service (`api/`)

Same route surface and behavior as the Rocket service:

- `GET|POST /tw/id/{user_id}`, `GET|POST /tw/{screen_name_query}` — account
  history lookups (cookie authorization for GET, GitHub token form field for
  POST; results date-limited for untrusted callers, inclusions exempted)
- `GET /tw/util/snowflake/{id}` — Snowflake timestamp decoding
- `GET /login/status`, `GET /logout`
- `GET /login/{github,google,twitter}` and `GET /auth/{github,google,twitter}` —
  OAuth flows (GitHub and Google via OAuth 2.0 with a private state cookie for
  CSRF protection, Twitter via OAuth 1.0a)

Tokens are stored in the same SQLite schema as before (an existing database
works unchanged; a missing one is created on startup). Access levels still come
from the authorizations CSV file. Token cookies keep their legacy names
(`github_token`, `google_token`, `twitter_token`) but are encrypted with this
service's `secret_key`, so existing browser sessions will re-authenticate once.

Configuration lives in a TOML file (path as the first CLI argument, default
`Api.toml`; see `api/Api.toml.example`), with `MEMORY_LOL_`-prefixed
environment variable overrides for secrets, e.g. `MEMORY_LOL_SECRET_KEY` and
`MEMORY_LOL_GITHUB__CLIENT_SECRET`.

```bash
cd next
cargo run --release -p memory-lol-api -- api/Api.toml
```

Behavioral notes:

- Token inserts are idempotent (`REPLACE` instead of `INSERT`), so a provider
  re-issuing an unchanged access token no longer causes a 500.
- The Twitter flow still stores the consumer key as the token value (inherited
  from the shared `auth` crate's schema semantics).

## Frontend (`app/`)

A SvelteKit single-page app served statically under `/app`, functionally
equivalent to the CRA/Bulma app: search by screen name (including
comma-separated lists and `prefix*` queries) or Twitter ID, results tables with
first/last-seen dates and external links, provider sign-in status, and
post-login return to the page the user started from.

```bash
cd next/app
npm install
npm run dev      # dev server; proxies /v1 to 127.0.0.1:8000
npm run build    # static site in build/, deploy under /app
npm run check    # svelte-check
```

The API root defaults to the page origin with a `/v1` prefix (matching the
production reverse proxy and the dev proxy); set `PUBLIC_API_ROOT` at build
time to point elsewhere.
