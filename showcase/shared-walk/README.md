# Shared Walk

The og.shared demo (guide `shared-worlds`, docs/design.md §2.12): every
visitor is a capsule on a grey field, and every other visitor near it is
drawn as another capsule that the engine moves. With no keys held, a
visitor walks a circle of its own, so headless clients see each other
move. WASD steers. The game names no server and no token: the host
supplies the connection.

## Self-hosted, with no platform

```sh
head -c 32 /dev/urandom | base64 > secret
og world-server --auth hs256 --secret-file secret --audience shared-walk

# native: a static session provider (one endpoint, one token)
og world-server token --secret-file secret --audience shared-walk --sub alice --name Alice > alice.token
og run . --shared ws://127.0.0.1:47910/ --shared-token-file alice.token

# web: the page's tokenEndpoint provider asks tools/serve.ts, which signs with the same secret
og build .
bun tools/serve.ts --secret-file secret --port 8080 --server ws://127.0.0.1:47910/
```

For local work without tokens, run `og world-server --auth none --dev` and
`og run . --shared ws://127.0.0.1:47910/`.

## Hosted elsewhere

The game code doesn't change. A page names another session provider
(`window.__OG_PROVIDER.sessions`, a platform's Player), and the server
verifies that issuer's tokens (`--auth jwks --jwks <url> --audience <game>`).

`packages/engine/tests/web-shared.test.ts` runs two pages and a page
without a provider against a self-hosted server.
