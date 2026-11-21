// Minisign verification, the one implementation for both callers: the
// launcher (`packages/launcher/og.ts`, whose installers carry this file
// beside it) and `og vfx add` (`src/node/vfx-pack.ts`). It verifies the
// release archives' `SHA256SUMS.minisig` (docs/release.md §1) and a vfx
// pack's signature (`docs/vfx-packs.md §5`).

import { createHash, createPublicKey, verify } from "node:crypto";

// The node members this file uses (the package type-checks without node's declarations: src/env.d.ts).
declare module "node:crypto" {
  interface KeyObject { readonly type: string }
  export function createPublicKey(o: { key: Uint8Array; format: "der"; type: "spki" }): KeyObject;
  export function verify(algorithm: null, data: Uint8Array, key: KeyObject, signature: Uint8Array): boolean;
}

/** The engine's release key (docs/release.md §1; the owner, 2026-11-05). */
export const ENGINE_PUBKEY = "RWSJfZkSrBHM9kOnWQNC0WoGXEFI+9NMCj8+Wy6WJDkYl9pgPGkoCoxt";
export const DOWNLOADS = "https://origingame.ai/downloads/engine";

/** A signature that did not verify; the message names what mismatched. */
export class MinisignError extends Error {
  readonly code = "signature";
}

const hexBytes = (hex: string) => Uint8Array.from(hex.match(/../g) ?? [], (h) => Number.parseInt(h, 16));
const latin1 = (b: Uint8Array) => String.fromCharCode(...b);
const base64 = (s: string): Uint8Array => {
  try {
    return Uint8Array.from(atob(s.trim()), (c) => c.charCodeAt(0));
  } catch {
    return new Uint8Array(0);
  }
};
const same = (a: Uint8Array, b: Uint8Array) => a.length === b.length && a.every((v, i) => v === b[i]);
function concat(parts: Uint8Array[]): Uint8Array {
  const out = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  let at = 0;
  for (const p of parts) {
    out.set(p, at);
    at += p.length;
  }
  return out;
}
const keyId = (b: Uint8Array) => [...b].reverse().map((v) => v.toString(16).padStart(2, "0")).join("").toUpperCase();

const ED25519_SPKI = hexBytes("302a300506032b6570032100");

/**
 * Verifies a minisign signature (`minisign -S`'s .minisig: the prehashed
 * `ED` algorithm, BLAKE2b-512 of the file, or the legacy `Ed`) of `data`
 * with a public key line, including the trusted comment's global
 * signature. Returns the trusted comment; throws MinisignError on any
 * mismatch.
 */
export function verifyMinisign(data: Uint8Array, signature: string, pubkey: string): string {
  const pk = base64(pubkey);
  if (pk.length !== 42 || latin1(pk.subarray(0, 2)) !== "Ed") fail("the engine's public key is not a minisign Ed25519 key");
  const lines = signature.split(/\r?\n/);
  const sig = base64(lines[1] ?? "");
  const trusted = lines[2] ?? "";
  const global = base64(lines[3] ?? "");
  if (sig.length !== 74 || !trusted.startsWith("trusted comment: ") || global.length !== 64) fail("SHA256SUMS.minisig is not a minisign signature");
  const alg = latin1(sig.subarray(0, 2));
  if (alg !== "ED" && alg !== "Ed") fail(`SHA256SUMS.minisig uses the unknown algorithm ${alg}`);
  if (!same(sig.subarray(2, 10), pk.subarray(2, 10))) fail(`SHA256SUMS.minisig is signed by key ${keyId(sig.subarray(2, 10))}, not the engine's ${keyId(pk.subarray(2, 10))}`);
  const key = createPublicKey({ key: concat([ED25519_SPKI, pk.subarray(10)]), format: "der", type: "spki" });
  const message = alg === "ED" ? hexBytes(createHash("blake2b512").update(data).digest("hex")) : data;
  if (!verify(null, message, key, sig.subarray(10))) fail("SHA256SUMS.minisig does not verify: the file was changed or not signed by the engine's key");
  const comment = trusted.slice("trusted comment: ".length);
  if (!verify(null, concat([sig.subarray(10), new TextEncoder().encode(comment)]), key, global)) fail("SHA256SUMS.minisig's trusted comment does not verify");
  return comment;
}

function fail(why: string): never {
  throw new MinisignError(why);
}
