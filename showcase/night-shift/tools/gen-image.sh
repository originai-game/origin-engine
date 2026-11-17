#!/usr/bin/env bash
# One image from the origin gateway, journalled the moment the gateway
# answers: the response is kept whole under .og/gateway/, its id and URL are
# appended to tools/gateway-journal.md, and only then is the image written.
#
#   tools/gen-image.sh <name> <prompt-file> <out.png> [model]
#
# The key is $OG_PLATFORM_DEVELOPMENT_KEY. A call that ends without an
# answer is journalled as unknown and never retried by this script.
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
name="$1"; prompt_file="$2"; out="$3"; model="${4:-gpt-image-2.6-sunburst}"
gateway="${OG_GATEWAY_URL:-https://api.origingame.ai}"
journal="$here/tools/gateway-journal.md"
raw="$here/.og/gateway"
mkdir -p "$raw" "$(dirname "$out")"
response="$raw/$name.$(date -u +%Y%m%dT%H%M%S).json"
when="$(date -u '+%Y-%m-%d %H:%M')"
body="$(jq -n --arg model "$model" --rawfile prompt "$prompt_file" '{model: $model, prompt: $prompt}')"
status="$(curl -sS -m 600 -o "$response" -w '%{http_code}' "$gateway/v1/images/generations" \
  -H "Authorization: Bearer $OG_PLATFORM_DEVELOPMENT_KEY" -H 'Content-Type: application/json' -d "$body" || true)"
url="$(jq -r '.media_url // .data[0].url // empty' "$response" 2>/dev/null || true)"
generation="$(jq -r '.data[0].generation_id // empty' "$response" 2>/dev/null || true)"
inline="$(jq -r 'if (.data[0].b64_json // "") != "" then "yes" else empty end' "$response" 2>/dev/null || true)"
if [[ -z $url && -z $inline ]]; then
  printf '| %s | generate_image (%s) | %s | none | HTTP %s, no image: outcome unknown, not retried (%s) |\n' \
    "$when" "$model" "$name" "$status" "${response#"$here"/}" >> "$journal"
  echo "gen-image: $name: HTTP $status, no image (journalled; see ${response#"$here"/})" >&2
  exit 1
fi
if [[ -n $url ]]; then
  media="$(sed -E 's#.*/media/([0-9a-f]+)/.*#\1#' <<< "$url")"
  printf '| %s | generate_image (%s) | %s | generation %s | media %s (png) |\n' "$when" "$model" "$name" "$generation" "$media" >> "$journal"
  curl -sSL -m 300 -o "$out" "$url"
else
  # The gateway's own API answers the bytes, with no stored media: the image is the response.
  jq -r '.data[0].b64_json' "$response" | base64 -d > "$out"
  printf '| %s | generate_image (%s) | %s | generation %s | bytes in the response, no media URL; sha256 %s |\n' \
    "$when" "$model" "$name" "$generation" "$(sha256sum "$out" | cut -d' ' -f1)" >> "$journal"
fi
echo "gen-image: $name -> $out ($(stat -c %s "$out") bytes)"
