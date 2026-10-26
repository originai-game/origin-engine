# The executor's platform for the install test (docs/release.md §3):
# node 22 on Debian bookworm, bun beside it, nothing of the engine. The
# test installs the packages into it from the tarballs or the registry.
FROM node:22-bookworm-slim
COPY --from=oven/bun:1.4.4 /usr/local/bin/bun /usr/local/bin/bun
RUN ln -s /usr/local/bin/bun /usr/local/bin/bunx \
  && apt-get update \
  && apt-get install -y --no-install-recommends ca-certificates git \
  && rm -rf /var/lib/apt/lists/*
ENV CI=1
