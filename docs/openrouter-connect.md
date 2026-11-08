# OpenRouter Connect

First-party OpenRouter OAuth PKCE flow. The CLI and server-side flow are implemented
in `src/openrouter_connect.rs` ([documented flow](https://openrouter.ai/docs/guides/overview/auth/oauth)).
The web UI uses the official TypeScript SDK `@openrouter/sdk` in the browser (below).
It replaces pasting an `OPENROUTER_API_KEY`.

## Browser (web UI, via `@openrouter/sdk`)

Menu -> "Connect OpenRouter" (or the welcome page button, which opens
`/ui/?or_connect=start`). `website/src/openrouterConnect.ts` (SDK loaded by dynamic
import, code-split from the main bundle):

1. `oAuth.createSHA256CodeChallenge()`; the verifier is kept in `sessionStorage`.
2. Redirect to the SDK-built auth URL, callback `${origin}/ui/?or_connect=1`.
3. On return, the SDK exchanges the code in the browser, the key is `POST`ed to
   `POST /api/openrouter/key` `{key}` (write route: admin token in shared-write
   mode), then the verifier is cleared and the query stripped. The key is never
   logged or rendered.
4. The daemon stores it in the 0600 key file and answers
   `{ok, restart_required: true}`: keys are read once at startup, so restart the
   daemon (the UI says so).

Accounts with "require key expiration" must turn it off for this flow: OAuth keys
carry no expiry (the SDK has no option for it).

## CLI (local)

```bash
kurultai connect openrouter            # opens the browser; prints the URL too
kurultai connect openrouter --no-open  # print the URL only
kurultai connect openrouter --port 8765
```

Binds `127.0.0.1` (ephemeral port unless `--port`), waits for
`GET /callback?code&state`, exchanges the code (S256 PKCE, `state` verified,
single use, 10 min TTL) and stores the key. `kurultai connect <instance-url>`
(device flow for Kurultai instances) is unchanged.

## Hosted / solo daemon

1. `POST /api/openrouter/connect` returns `{"auth_url": "...", "expires_in": 600}`.
   It is a write route: in shared-write mode it needs the admin bearer token.
   The callback URL is `$KURULTAI_PUBLIC_URL` (or the request's human base URL)
   plus `/api/openrouter/callback`.
2. Open `auth_url` in a browser and approve. OpenRouter redirects to
   `GET /api/openrouter/callback`, which has no bearer (browser redirect) and is
   gated by the single-use, short-lived `state`; the PKCE verifier exists only in
   daemon memory.
3. Restart the daemon so it re-reads the key.

## Key storage

- `<config-dir>/openrouter.key` (or `KURULTAI_API_KEY_FILE`), mode 0600, atomic
  write. This is the file `api_key_from_keyfile` already reads, so embeddings and
  `ask` pick it up. `OPENROUTER_API_KEY` / `KURULTAI_API_KEY` env still wins.
- CLI additionally copies to the `omaseal` keyring (`kurultai/openrouter`) when installed.
- Never written to `config.toml` or logs; `Debug` output is redacted.

## Expiry

OpenRouter's documented exchange body is only `code`, `code_verifier`,
`code_challenge_method`: there is **no expiry/name/limit parameter**, so Kurultai
cannot request an expiring key (it sends `key_label=Kurultai` on the auth URL).
Accounts that require expiring keys answer 401 "API keys without an expiration
date are not allowed". The exchange, embeddings and `ask` map that to an
actionable message: re-run `kurultai connect openrouter` or create a key with an
expiration at <https://openrouter.ai/settings/keys>.
